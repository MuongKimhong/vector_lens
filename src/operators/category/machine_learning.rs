use bevy::prelude::*;
use makara::prelude::*;
use polars::prelude::{col, NamedFrom, Series, lit, DataFrameOps,
    CsvReadOptions, CsvWriter, DataFrame, SerReader, SerWriter, IntoLazy,
    datatypes::{Float64Type, Float32Type}, IndexOrder
};
use linfa::prelude::*;
use linfa_linear::{LinearRegression, FittedLinearRegression};
use ndarray::ArrayBase;
use polars::datatypes::DataType;
use std::collections::HashMap;
use crate::utils::*;
use super::*;

// Normalizer & Encoder operator that applies transformations based on data types
pub fn normalizer_and_encoder_operator() -> Operator {
    Operator::new(
        "Normalizer & Encoder",
        OperatorKind::NormalizerAndEncoder,
        DataValue::Table(DataFrame::empty()),
        DataValue::Table(DataFrame::empty()),
        OperatorCategory::MachineLearning,
        HashMap::from([
            ("normalization_method".to_string(), PropertyValue::String("min-max".to_string())),
            ("apply_normalization".to_string(), PropertyValue::Bool(true)),
            ("apply_one_hot_encoding".to_string(), PropertyValue::Bool(true)),
        ])
    )
}

pub fn handle_normalizer_and_encoder_operator_execution(
    task_sender: &Sender<TaskChannelEvent>,
    input: &DataValue,
    properties: &HashMap<String, PropertyValue>
) -> DataValue {
    let df = match input {
        DataValue::Table(df) => df,
        _ => return DataValue::Error,
    };

    let normalization_method = match properties.get("normalization_method") {
        Some(PropertyValue::String(method)) => method.clone(),
        _ => "min-max".to_string()
    };

    let apply_norm = match properties.get("apply_normalization") {
        Some(PropertyValue::Bool(b)) => *b,
        _ => true,
    };
    let apply_encode = match properties.get("apply_one_hot_encoding") {
        Some(PropertyValue::Bool(b)) => *b,
        _ => true,
    };

    let mut normalization_exprs = Vec::new();
    let schema = df.schema();

    if apply_norm {
        for name in df.get_column_names() {
            if let Some(dtype) = schema.get(name) {
                if dtype.is_numeric() {
                    let name_str = name.as_str();

                    let e = match normalization_method.as_str() {
                        "z-score" => {
                            // (x - mean) / std
                            (col(name_str) - col(name_str).mean()) / (col(name_str).std(1) + lit(1e-8))
                        },
                        "min-max" | _ => {
                            // (x - min) / (max - min)
                            let min = col(name_str).min();
                            let max = col(name_str).max();
                            (col(name_str) - min.clone()) / (max - min + lit(1e-8))
                        }
                    };

                    normalization_exprs.push(e.alias(name_str));
                }
            }
        }
    }

    // df with just normalized numeric columns
    let normalized_df = match df.clone().lazy().select(normalization_exprs).collect() {
        Ok(res) => res,
        Err(e) => {
            let _ = task_sender.send(
                TaskChannelEvent::LogMessage(log_error(&format!("Norm Error: {}", e)))
            );
            return DataValue::Error;
        }
    };

    let _ = task_sender.send(TaskChannelEvent::LogMessage(
        log_normal("Numeric columns normalized")
    ));

    let mut encoded_df = DataFrame::empty();

    if apply_encode {
        let categorical_df = df.select(
            df.get_column_names().into_iter()
                .filter(|name| {
                    let dtype = df.column(name).unwrap().dtype();
                    dtype.is_categorical() || dtype.is_string()
                })
        );

        match categorical_df {
            Ok(df) => {
                encoded_df = match df.to_dummies(None, true, true) {
                    Ok(dummy_df) => dummy_df,
                    Err(e) => {
                        let _ = task_sender.send(TaskChannelEvent::LogMessage(
                            log_error(&format!("Encoding Error: {}", e))
                        ));
                        return DataValue::Error;
                    }
                };
            }
            Err(e) => {
                let _ = task_sender.send(TaskChannelEvent::LogMessage(
                    log_error(&format!("Encoding Error: {}", e))
                ));
                return DataValue::Error;
            }
        }
    }

    let final_df = match encoded_df.hstack(normalized_df.columns()) {
        Ok(df) => df,
        Err(e) => {
            let _ = task_sender.send(TaskChannelEvent::LogMessage(
                log_error(&format!("HStack Error: {}", e))
            ));
            return DataValue::Error;
        }
    };

    let _ = task_sender.send(TaskChannelEvent::LogMessage(
        log_normal("Categorical & String columns encoded")
    ));
    DataValue::Table(final_df)
}

// This function would be called by the UI to create visualization
// and would use the existing task system to send events
// pub fn handle_update_normalizer_and_encoder_property_after_op_spawned(
//     mut messages: MessageReader<UpdateNormalizerAndEncoderPropertyAfterOPSpawned>,
//     mut operator_q: Query<&mut Operator>,
//     mut column_q: ColumnQuery,
//     mut commands: Commands,
//     pre_read_content: Res<PreReadCsvOrExcelContent>,
// ) {
//     for _msg in messages.read() {
//         if *pre_read_content.get() == DataValue::None {
//             continue;
//         }

//         match pre_read_content.get() {
//             DataValue::Table(df) => {

//             }
//             _ => {}
//         }
//     }
// }

pub fn train_test_split_operator() -> Operator {
    Operator::new(
        "Train & Test split",
        OperatorKind::TrainTestSplit,
        DataValue::Table(DataFrame::empty()),
        DataValue::Table(DataFrame::empty()),
        OperatorCategory::MachineLearning,
        HashMap::from([
            ("train_set_percent".to_string(), PropertyValue::Float(80.0)),
            ("shuffle".to_string(), PropertyValue::Bool(true)),
        ])
    )
}

pub fn handle_train_test_split_operator_execution(
    task_sender: &Sender<TaskChannelEvent>,
    input: &DataValue,
    properties: &HashMap<String, PropertyValue>
) -> DataValue {
    let _ = task_sender.send(TaskChannelEvent::LogMessage(
        log_normal("[Train & Test split] Started train & test split")
    ));

    let train_set_percent = match properties.get("train_set_percent") {
        Some(PropertyValue::Float(percent)) => percent,
        _ => {
            let _ = task_sender.send(TaskChannelEvent::LogMessage(
                log_error("[Train & Test split] train_set_percent property missing")
            ));
            return DataValue::Error;
        }
    };

    let shuffle = match properties.get("shuffle") {
        Some(PropertyValue::Bool(flag)) => flag,
        _ => {
            let _ = task_sender.send(TaskChannelEvent::LogMessage(
                log_error("[Train & Test split] shuffle property missing")
            ));
            return DataValue::Error;
        }
    };

    let df = match input {
        DataValue::Table(df) => df,
        _ => {
            let _ = task_sender.send(TaskChannelEvent::LogMessage(
                log_error("[Train & Test split] Input is not a Table")
            ));
            return DataValue::Error;
        }
    };
    let total_rows = df.height();

    if total_rows == 0 {
        return DataValue::Table(df.clone());
    }

    let new_df;

    if *shuffle {
        new_df = match df.sample_frac(
            &Series::new("frac".into(), &[1.0f64]),
            false,
            true,
            Some(42)
        ) {
            Ok(shuffled) => shuffled,
            Err(_) => return DataValue::Error, // Handle potential sampling errors
        };
    }
    else {
        new_df = df.clone();
    }

    let total_rows = df.height();
    let train_size = (total_rows as f64 * (train_set_percent / 100.0) as f64) as usize;
    let test_size = total_rows - train_size;
    let train_df = new_df.slice(0, train_size);
    let test_df = new_df.slice(train_size as i64, test_size);
    let _ = task_sender.send(TaskChannelEvent::SetTestData(DataValue::Table(test_df)));

    DataValue::Table(train_df)
}

pub fn linear_regression_operator() -> Operator {
    Operator::new(
        "Linear Regression",
        OperatorKind::LinearRegression,
        DataValue::Table(DataFrame::empty()),
        DataValue::Table(DataFrame::empty()),
        OperatorCategory::MachineLearning,
        HashMap::from([
            ("target".to_string(), PropertyValue::String("".to_string()))
        ])
    )
}

fn on_select_target_change(
    change: On<Change<String>>,
    mut operator_q: Query<&mut Operator>
) {
    for mut op in operator_q.iter_mut() {
        let Some(target_property) = op.properties.get_mut("target") else {
            continue;
        };

        match target_property {
            PropertyValue::String(target) => {
                if !change.data.trim().is_empty() {
                    *target = change.data.clone();
                }
            }
            _ => {}
        }
    }
}

fn handle_spawwn_select_widget_for_linear_regression(
    operator_q: &mut Query<&mut Operator>,
    column_q: &mut ColumnQuery,
    commands: &mut Commands,
    pre_read_content: &Res<PreReadCsvOrExcelContent>
) {
    match pre_read_content.get() {
        DataValue::Table(df) => {
            let columns: Vec<String> = df.get_column_names()
                .iter()
                .map(|ps| ps.to_string())
                .collect();

            for _op in operator_q.iter() {
                let mut choices = Vec::new();

                let Some(column) = column_q.find_by_id("linear-regression-column") else {
                    return;
                };
                commands.entity(column.entity).despawn_children();
                commands.entity(column.entity).with_children(|parent| {
                    for column in columns.iter() {
                        choices.push(column.to_string());
                    }
                    let temp_choices: Vec<&str> = choices.iter().map(|s| s.as_str()).collect();

                    parent.spawn(select_!(
                        "Select target",
                        choices: &temp_choices,
                        width: percent(100),
                        on: on_select_target_change
                    ));
                });
            }
        }
        _ => {}
    }
}

pub fn handle_update_select_widget_on_select_attribute_change(
    mut msg: MessageReader<UpdateLinearRegressionTargetChoice>,
    mut operator_q: Query<&mut Operator>,
    mut column_q: ColumnQuery,
    mut commands: Commands
) {
    for msg in msg.read() {
        let columns: Vec<String> = msg.0
            .iter()
            .filter_map(|i| {
                match i {
                    PropertyValue::String(v) => Some(v.clone()), // Return Some(String)
                    _ => None,                                   // Return None to filter out
                }
            })
            .collect();

        for _op in operator_q.iter() {
            let mut choices = Vec::new();

            let Some(column) = column_q.find_by_id("linear-regression-column") else {
                return;
            };
            commands.entity(column.entity).despawn_children();
            commands.entity(column.entity).with_children(|parent| {
                for column in columns.iter() {
                    choices.push(column.to_string());
                }
                let temp_choices: Vec<&str> = choices.iter().map(|s| s.as_str()).collect();

                parent.spawn(select_!(
                    "Select target",
                    choices: &temp_choices,
                    width: percent(100),
                    on: on_select_target_change
                ));
            });
        }
    }
}

pub fn handle_update_linear_regression_property_after_op_spawned(
    mut messages: MessageReader<UpdateLinearRegressionPropertyAfterOPSpawned>,
    mut operator_q: Query<&mut Operator>,
    mut column_q: ColumnQuery,
    mut commands: Commands,
    pre_read_content: Res<PreReadCsvOrExcelContent>,
) {
    for _msg in messages.read() {
        if *pre_read_content.get() == DataValue::None {
            continue;
        }

        handle_spawwn_select_widget_for_linear_regression(
            &mut operator_q,
            &mut column_q,
            &mut commands,
            &pre_read_content
        );
    }
}

pub fn handle_update_linear_regression_property_on_precontent_change(
    mut operator_q: Query<&mut Operator>,
    mut column_q: ColumnQuery,
    mut commands: Commands,
    pre_read_content: Res<PreReadCsvOrExcelContent>,
) {
    if !pre_read_content.is_changed() {
        return;
    }

    if *pre_read_content.get() == DataValue::None {
        return;
    }

    handle_spawwn_select_widget_for_linear_regression(
        &mut operator_q,
        &mut column_q,
        &mut commands,
        &pre_read_content
    );
}

pub fn handle_linear_regression_operator_execution(
    task_sender: &Sender<TaskChannelEvent>,
    input: &DataValue,
    properties: &std::collections::HashMap<String, PropertyValue>
) -> DataValue {
    let _ = task_sender.send(
        TaskChannelEvent::LogMessage(log_normal("[LR] Starting LR process..."))
    );

    // 1. Get DataFrame
    let df = match input {
        DataValue::Table(df) => df,
        _ => {
            let _ = task_sender.send(
                TaskChannelEvent::LogMessage(log_error("[LR] Input is not a table"))
            );
            return DataValue::Error;
        }
    };

    // 2. Extract Target Column Name
    let target_name = match properties.get("target") {
        Some(PropertyValue::String(s)) if !s.is_empty() => s,
        _ => {
            let _ = task_sender.send(
                TaskChannelEvent::LogMessage(log_error("[LR] Target column not specified"))
            );
            return DataValue::Error;
        }
    };

    // 3. Dynamically Get Feature Names (Everything numeric except the target)
    let target_clean = target_name.trim() // Clean any accidental whitespace
        .replace("\r", "")
        .replace("\"", "");

    let feature_names: Vec<&str> = df.get_column_names()
        .into_iter()
        .map(|c| c.as_str())
        .filter(|&col_name| col_name.trim() != target_clean) // Strictly filter out target
        .collect();

    if feature_names.is_empty() {
        let _ = task_sender.send(
            TaskChannelEvent::LogMessage(log_error("[LR] No valid feature columns found"))
        );
        return DataValue::Error;
    }
    // --- 4 & 5. EXTRACT FEATURES AND TARGET IN PARALLEL USING RAYON ---
    println!("[LR] Extracting X and Y matrices in parallel...");

    // rayon::join takes two closures and runs them concurrently on Rayon's thread pool
    let (x_flat_vec, y_flat_vec) = rayon::join(
        || {
            // Task A: Extract and flatten features
            let x_df = df
                .select(&feature_names)
                .expect("[LR] Feature selection failed");

            x_df.to_ndarray::<Float32Type>(IndexOrder::C)
                .expect("[LR] Failed to flatten features")
                .into_raw_vec()
        },
        || {
            // Task B: Extract and flatten target
            let y_df = df
                .select(&[&target_clean as &str])
                .expect("[LR] Target selection failed");

            y_df.to_ndarray::<Float32Type>(IndexOrder::C)
                .expect("[LR] Failed to flatten target")
                .into_raw_vec()
        }
    );

    // --- 6. INITIALIZE DATASET DIRECTLY FROM PLAN VECS ---
    // Linfa allows converting a 1D vector into a 2D dataset space using a flat array shape
    let num_rows = df.height();
    let num_features = feature_names.len();

    // Use linfa's native macro/structures to build the matrices using its internal dependency version
    let records = ArrayBase::from_shape_vec((num_rows, num_features), x_flat_vec).unwrap();
    let targets = ArrayBase::from_shape_vec(num_rows, y_flat_vec).unwrap();

    let dataset = Dataset::new(records, targets);

    // --- 7. TRAIN THE MODEL ---
    let _ = task_sender.send(TaskChannelEvent::LogMessage(
        log_normal("[LR] Training model...")
    ));

    let model: FittedLinearRegression<f32> = match LinearRegression::default().fit(&dataset) {
        Ok(m) => m,
        Err(e) => {
            let _ = task_sender.send(
                TaskChannelEvent::LogMessage(log_error(
                    &format!("[LR] Training failed: {e}")
                ))
            );
            return DataValue::Error;
        }
    };

    let weights = model.params();
    let intercept = model.intercept();

    let _ = task_sender.send(TaskChannelEvent::LogMessage(
        log_normal(&format!("[LR] Model Intercept (Baseline): {:.4}", intercept))
    ));

    for (i, col_name) in feature_names.iter().enumerate() {
        if let Some(weight_val) = weights.get(i) {
            println!("[LR] Feature '{}' Weight: {:.4}", col_name, weight_val);
        }
    }

    let _ = task_sender.send(
        TaskChannelEvent::LogMessage(
            log_normal("[LR] Model successfully trained, making prediction on test set..")
        )
    );

    let data = LinearRegressionPredictionData {
        feature_names: feature_names.iter().map(|f| f.to_string()).collect(),
        target_name: target_clean.clone(),
        model: model.clone()
    };

    let _ = task_sender.send(
        TaskChannelEvent::LinearRegressionPrediction(data)
    );

    DataValue::None
}
