use bevy::prelude::*;
use makara::prelude::*;
use polars::chunked_array::ops::FillNullStrategy;
use polars::prelude::{col, lit, IntoLazy};
use crate::io::handle_update_rmv_property_on_get_pre_read_content;
use crate::utils::*;
use super::*;

// TODO on replace missing value op
//
// 1. Create a resource to hold dataframe of read csv and read excel.
//    When user set file path property in these 2 operators, spawn
//    a background task to pre read the content and put into the resource.
//
// 2. Whenever user add replace missing value op, send a message to go thru content and
//    get the empty string and null columns and add those columns into
//    "columns_with_missing_value" where key is column name and value
//    is what to replace with.
//
//
pub fn replace_missing_value_operator() -> Operator {
    Operator::new(
        "Replace missing value",
        OperatorKind::ReplaceMissingValue,
        DataValue::Table(DataFrame::empty()),
        DataValue::Table(DataFrame::empty()),
        OperatorCategory::Cleaning,
        HashMap::from([
            ("columns_with_missing_value".to_string(), PropertyValue::Map(HashMap::new())),
        ])
    )
}

pub fn handle_replace_missing_value_operator_execution(
    task_sender: &Sender<TaskChannelEvent>,
    input: &DataValue,
    properties: &HashMap<String, PropertyValue>
) -> DataValue {
    let _ = task_sender.send(TaskChannelEvent::LogMessage(
        log_normal("[Replace Missing Value] Started replacing missing values")
    ));

    // 1. Extract the DataFrame from the input
    let df = match input {
        DataValue::Table(df) => df,
        _ => {
            let _ = task_sender.send(TaskChannelEvent::LogMessage(
                log_error("[Replace Missing Value] Input is not a Table")
            ));
            return DataValue::Error;
        }
    };

    // 2. Extract the column-to-strategy map
    let column_map = match properties.get("columns_with_missing_value") {
        Some(PropertyValue::Map(map)) => map,
        _ => {
            let _ = task_sender.send(TaskChannelEvent::LogMessage(
                log_error("[Replace Missing Value] Configuration property missing")
            ));
            return DataValue::Error;
        }
    };

    let mut exprs = Vec::new();

    // 3. Build Expressions
    for (col_name, strategy_val) in column_map {
        if let PropertyValue::String(strategy) = strategy_val {

            // Validate column existence
            let series = match df.column(col_name) {
                Ok(s) => s,
                Err(_) => {
                    let _ = task_sender.send(TaskChannelEvent::LogMessage(
                        log_error(&format!("[Replace Missing Value] Column '{}' not found", col_name))
                    ));
                    return DataValue::Error;
                }
            };

            let dtype = series.dtype();
            let is_numeric = dtype.is_numeric();
            let col_expr = col(col_name);

            let filled_expr = match strategy.as_str() {
                "Null" | "" => continue,

                "Mean" | "Average" | "Max" | "Min" => {
                    if !is_numeric {
                        let _ = task_sender.send(TaskChannelEvent::LogMessage(
                            log_error(&format!(
                                "[Replace Missing Value] Cannot use '{}' on non-numeric column '{}'",
                                strategy,
                                col_name
                            ))
                        ));
                        return DataValue::Error;
                    }

                    match strategy.as_str() {
                        "Max" => col_expr.clone().fill_null(col_expr.max()),
                        "Min" => col_expr.clone().fill_null(col_expr.min()),
                        _ => col_expr.clone().fill_null(col_expr.mean()), // Mean or Average
                    }
                }

                "Forward Fill" => col_expr.fill_null_with_strategy(FillNullStrategy::Forward(None)),
                "Backward Fill" => col_expr.fill_null_with_strategy(FillNullStrategy::Backward(None)),
                "Unknown" => col_expr.fill_null(lit("Unknown")),

                custom_val => col_expr.fill_null(lit(custom_val)),
            };

            exprs.push(filled_expr);
        }
    }

    // 4. Collect and return
    let result = df.clone()
        .lazy()
        .with_columns(exprs)
        .collect();

    match result {
        Ok(new_df) => {
            let _ = task_sender.send(TaskChannelEvent::LogMessage(
                log_normal("[Replace Missing Value] Finished replacing values")
            ));
            DataValue::Table(new_df)
        }
        Err(e) => {
            let _ = task_sender.send(TaskChannelEvent::LogMessage(
                log_error(&format!("[Replace Missing Value] {e}"))
            ));
            DataValue::Error
        }
    }
}

pub fn handle_update_rmv_property_after_op_spawned(
    mut messages: MessageReader<UpdateReplaceMissingValuePropertyAfterOPSpawned>,
    mut operator_q: Query<&mut Operator>,
    mut column_q: ColumnQuery,
    mut commands: Commands,
    pre_read_content: Res<PreReadCsvOrExcelContent>,
) {
    for _msg in messages.read() {
        if *pre_read_content.get() == DataValue::None {
            continue;
        }

        match pre_read_content.get() {
            DataValue::Table(df) => {
                if let Ok(columns) = get_dirty_column_names(&df) {
                    handle_update_rmv_property_on_get_pre_read_content(
                        &columns,
                        &mut operator_q,
                        &mut column_q,
                        &mut commands
                    );
                }
            }
            _ => {}
        }
    }
}
