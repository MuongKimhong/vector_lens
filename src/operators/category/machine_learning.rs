use bevy::prelude::*;
use makara::prelude::*;
use polars::prelude::{col, NamedFrom, Series, lit, DataFrameOps, CsvReadOptions, CsvWriter, DataFrame, SerReader, SerWriter, IntoLazy};
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
            let _ = task_sender.send(TaskChannelEvent::LogMessage(log_error(&format!("Norm Error: {}", e))));
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

    println!("train df {:?}", train_df);
    println!("test df {:?}", test_df);

    let _ = task_sender.send(TaskChannelEvent::SetTestData(DataValue::Table(test_df)));

    DataValue::Table(train_df)
}
