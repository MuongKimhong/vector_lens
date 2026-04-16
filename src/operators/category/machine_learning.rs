use bevy::prelude::*;
use polars::prelude::{col, lit, DataFrameOps, CsvReadOptions, CsvWriter, DataFrame, SerReader, SerWriter, IntoLazy};
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
            ("selected_columns".to_string(), PropertyValue::List(Vec::new())),
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

    for name in df.get_column_names() {
        if let Some(dtype) = schema.get(name) {
            if dtype.is_numeric() && apply_norm {
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

    // df with just normalized numeric columns
    let mut normalized_df = match df.clone().lazy().select(normalization_exprs).collect() {
        Ok(res) => res,
        Err(e) => {
            let _ = task_sender.send(TaskChannelEvent::LogMessage(log_error(&format!("Norm Error: {}", e))));
            return DataValue::Error;
        }
    };

    let mut encoded_df = DataFrame::empty();

    if apply_encode {
        // 1. Select ONLY the columns that are String or Categorical
        let categorical_df = df.select(
            df.get_column_names().into_iter()
                .filter(|name| {
                    let dtype = df.column(name).unwrap().dtype();
                    dtype.is_categorical() || dtype.is_string()// Keeps Utf8, Categorical, Boolean, etc.
                })
        ).unwrap();

        // 2. Turn that subset into dummies
        encoded_df = match categorical_df.to_dummies(None, true, true) {
            Ok(dummy_df) => dummy_df,
            Err(e) => {
                let _ = task_sender.send(TaskChannelEvent::LogMessage(log_error(&format!("Encoding Error: {}", e))));
                return DataValue::Error;
            }
        };
    }
    println!("encoded df {:?}", encoded_df);
    println!("normalized df {:?}", normalized_df);

    let final_df = match encoded_df.hstack(normalized_df.columns()) {
        Ok(df) => df,
        Err(e) => {
            let _ = task_sender.send(TaskChannelEvent::LogMessage(log_error(&format!("HStack Error: {}", e))));
            return DataValue::Error;
        }
    };

    // // Success Logging
    // let _ = task_sender.send(TaskChannelEvent::LogMessage(
    //     log_normal(&format!("[TreyVisai] Success. Final width: {}", processed_df.width()))
    // ));

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
//     // Implementation for handling property updates when operator is spawned
//     // This would be similar to other property update handlers
//     for _msg in messages.read() {
//         if *pre_read_content.get() == DataValue::None {
//             continue;
//         }

//         match pre_read_content.get() {
//             DataValue::Table(df) => {
//                 // Handle property updates based on pre-read content
//                 // This would populate columns for selection properties
//             }
//             _ => {}
//         }
//     }
// }
