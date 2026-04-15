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
            ("normalization_method".to_string(), PropertyValue::String("z-score".to_string())),
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
    // 1. Get the DataFrame
    let df = match input {
        DataValue::Table(df) => df,
        _ => return DataValue::Error,
    };

    let apply_norm = match properties.get("apply_normalization") {
        Some(PropertyValue::Bool(b)) => *b,
        _ => true,
    };
    let apply_encode = match properties.get("apply_one_hot_encoding") {
        Some(PropertyValue::Bool(b)) => *b,
        _ => true,
    };

    // --- STEP 1: MATH (Lazy) ---
    let mut exprs = Vec::new();
    let schema = df.schema();

    for name in df.get_column_names() {
        if let Some(dtype) = schema.get(name) {
            if dtype.is_numeric() && apply_norm {
                // Standard Z-score
                let name = name.as_str();
                let e = (col(name) - col(name).mean()) / (col(name).std(1) + lit(1e-8));
                exprs.push(e.alias(name));
            }
        }
    }

    // Collect the math changes
    let mut processed_df = match df.clone().lazy().with_columns(exprs).collect() {
        Ok(res) => res,
        Err(e) => {
            let _ = task_sender.send(TaskChannelEvent::LogMessage(log_error(&format!("Norm Error: {}", e))));
            return DataValue::Error;
        }
    };

    // --- STEP 2: ENCODING (Eager) ---
    if apply_encode {
        // Correct signature: to_dummies(self, separator: Option<&str>, drop_first: bool, drop_nulls: bool)
        // This will automatically target all String/Categorical columns.
        processed_df = match processed_df.to_dummies(None, false, true) {
            Ok(dummy_df) => dummy_df,
            Err(e) => {
                let _ = task_sender.send(TaskChannelEvent::LogMessage(log_error(&format!("Encoding Error: {}", e))));
                return DataValue::Error;
            }
        };
    }

    // Success Logging
    let _ = task_sender.send(TaskChannelEvent::LogMessage(
        log_normal(&format!("[TreyVisai] Success. Final width: {}", processed_df.width()))
    ));

    DataValue::Table(processed_df)
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
