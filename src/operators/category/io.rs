use crossbeam_channel::Sender;

use crate::utils::create_log_with_timestamp;
use super::*;

pub fn read_csv_operator() -> Operator {
    Operator::new(
        "Read CSV",
        OperatorKind::ReadCSV,
        DataValue::None,
        DataValue::Table(DataFrame::empty()),
        OperatorCategory::IO,
        HashMap::from([
            ("file_path".to_string(), PropertyValue::String("".to_string()))
        ])
    )
}

pub fn handle_read_csv_operator_execution(
    task_sender: &Sender<TaskChannelEvent>,
    properties: &HashMap<String, PropertyValue>
) -> DataValue {
    let _ =task_sender.send(TaskChannelEvent::LogMessage(
        LogType::Normal(create_log_with_timestamp("[Read CSV] Started reading CSV file"))
    ));

    let path = match properties.get("path") {
        Some(PropertyValue::String(s)) => s,
        _ => {
            let _ = task_sender.send(TaskChannelEvent::LogMessage(
                LogType::Error(create_log_with_timestamp("[Read CSV] Failed to read CSV file"))
            ));
            return DataValue::None;
        }
    };

    let df = CsvReadOptions::default()
        .with_has_header(true)
        .with_infer_schema_length(None)
        .with_low_memory(true)
        .try_into_reader_with_file_path(Some(path.into()))
        .and_then(|reader| reader.finish());

    match df {
        Ok(frame) => {
            let _ = task_sender.send(TaskChannelEvent::LogMessage(
                LogType::Normal(create_log_with_timestamp("[Read CSV] Finished reading CSV file"))
            ));
            return DataValue::Table(frame)
        },
        Err(e) => {
            let _ = task_sender.send(TaskChannelEvent::LogMessage(
                LogType::Error(create_log_with_timestamp(&format!("[Read CSV] {e}")))
            ));
            DataValue::None
        }
    }
}
