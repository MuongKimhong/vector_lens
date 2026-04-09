use crossbeam_channel::Sender;
use polars::prelude::{CsvReadOptions, CsvWriter, DataFrame, SerReader, SerWriter};
use polars_excel_writer::PolarsExcelWriter;
use std::{fs::File, path::Path};

use crate::utils::{log_error, log_normal, log_success};
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
    let _ = task_sender.send(TaskChannelEvent::LogMessage(
        log_normal("[Read CSV] Started reading CSV file")
    ));

    let path = match properties.get("file_path") {
        Some(PropertyValue::String(s)) => s,
        _ => {
            let _ = task_sender.send(TaskChannelEvent::LogMessage(
                log_error("[Read CSV] Failed to read CSV file")
            ));
            return DataValue::Error;
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
                log_normal("[Read CSV] Finished reading CSV file")
            ));
            return DataValue::Table(frame)
        },
        Err(e) => {
            let _ = task_sender.send(TaskChannelEvent::LogMessage(
                log_error(&format!("[Read CSV] {e}"))
            ));
            DataValue::Error
        }
    }
}

pub fn read_excel_operator() -> Operator {
    Operator::new(
        "Read Excel",
        OperatorKind::ReadExcel,
        DataValue::None,
        DataValue::Table(DataFrame::empty()),
        OperatorCategory::IO,
        HashMap::from([
            ("file_path".to_string(), PropertyValue::String("".to_string()))
        ])
    )
}

pub fn handle_read_excel_operator_execution(
    task_sender: &Sender<TaskChannelEvent>,
    properties: &HashMap<String, PropertyValue>
) -> DataValue {

    // implemention goes here. It will be similar to handle_read_csv_operator_execution

    DataValue::None // just a dummy return, pls remove this.
}

pub fn save_csv_or_excel_operator() -> Operator {
    Operator::new(
        "Save to CSV or Excel",
        OperatorKind::SaveCSVOrExcel,
        DataValue::Table(DataFrame::empty()),
        DataValue::None,
        OperatorCategory::IO,
        HashMap::from([
            ("file_path".to_string(), PropertyValue::String("".to_string())),
            ("file_name".to_string(), PropertyValue::String("".to_string())),
            ("file_type".to_string(), PropertyValue::String("".to_string()))
        ])
    )
}

fn handle_write_to_csv_helper(
    input: &DataValue,
    task_sender: &Sender<TaskChannelEvent>,
    file: &mut File
) -> DataValue {
    match input {
        DataValue::Table(df) => {
            let mut df_clone = df.clone();
            println!("dataframe in save {:?}", df);

            match CsvWriter::new(file).include_header(true).with_separator(b',').finish(&mut df_clone) {
                Ok(_) => {
                    let _ = task_sender.send(TaskChannelEvent::LogMessage(
                        log_normal(&format!("[Save to Csv or Excel] Saved to Csv file"))
                    ));
                    return DataValue::None;
                }
                Err(e) => {
                    let _ = task_sender.send(TaskChannelEvent::LogMessage(
                        log_error(&format!("[Save to Csv or Excel] {e}"))
                    ));
                    return DataValue::Error;
                }
            }
        }
        _ => {
            let _ = task_sender.send(TaskChannelEvent::LogMessage(
                log_error(&format!("[Save to Csv or Excel] Failed to save to Csv"))
            ));
            return DataValue::Error;
        }
    }
}

fn handle_write_to_excel_helper(
    input: &DataValue,
    full_path: &str,
    task_sender: &Sender<TaskChannelEvent>
) -> DataValue {
    match input {
        DataValue::Table(df) => {
            let mut excel_writer = PolarsExcelWriter::new();

            if let Err(e) = excel_writer.write_dataframe(&df) {
                let _ = task_sender.send(TaskChannelEvent::LogMessage(
                    log_error(&format!("[Save to Csv or Excel] {e}"))
                ));
                return DataValue::Error;
            }

            if let Err(e) = excel_writer.save(full_path) {
                let _ = task_sender.send(TaskChannelEvent::LogMessage(
                    log_error(&format!("[Save to Csv or Excel] {e}"))
                ));
                return DataValue::Error;
            }

            return DataValue::None;
        }
        _ => {
            let _ = task_sender.send(TaskChannelEvent::LogMessage(
                log_error(&format!("[Save to Csv or Excel] Failed to save to Csv"))
            ));
            return DataValue::Error;
        }
    }
}

pub fn handle_save_csv_or_excel_operator_execution(
    task_sender: &Sender<TaskChannelEvent>,
    input: &DataValue,
    properties: &HashMap<String, PropertyValue>
) -> DataValue {
    let file_type = match properties.get("file_type") {
        Some(PropertyValue::String(s)) => s,
        _ => {
            let _ = task_sender.send(TaskChannelEvent::LogMessage(
                log_error("[Save to Csv or Excel] Invalid file type")
            ));
            return DataValue::Error;
        }
    };

    let file_path = match properties.get("file_path") {
        Some(PropertyValue::String(s)) => s,
        _ => {
            let _ = task_sender.send(TaskChannelEvent::LogMessage(
                log_error("[Save to Csv or Excel] Invalid destination")
            ));
            return DataValue::Error;
        }
    };

    let file_name = match properties.get("file_name") {
        Some(PropertyValue::String(s)) => s,
        _ => {
            let _ = task_sender.send(TaskChannelEvent::LogMessage(
                log_error("[Save to Csv or Excel] Invalid file name")
            ));
            return DataValue::Error;
        }
    };

    if file_name.is_empty() {
        let _ = task_sender.send(TaskChannelEvent::LogMessage(
            log_error("[Save to Csv or Excel] Invalid file name")
        ));
        return DataValue::Error;
    }

    let _ = task_sender.send(TaskChannelEvent::LogMessage(
        log_normal(&format!("[Save to Csv or Excel] Saving content to {file_type} file"))
    ));

    let extension = if file_type == "Csv" { ".csv" } else { ".xlsx" };
    let full_path = Path::new(file_path)
        .join(&format!("{file_name}{extension}"))
        .to_string_lossy()
        .into_owned();

    match std::fs::File::create(&full_path) {
        Ok(mut f) => {
            if extension == ".csv" {
                return handle_write_to_csv_helper(input, task_sender, &mut f);
            }
            else {
                return handle_write_to_excel_helper(input, &full_path, task_sender);
            }
        }
        Err(e) => {
            let _ = task_sender.send(TaskChannelEvent::LogMessage(
                log_error(&format!("[Save to Csv or Excel] {e}"))
            ));
            return DataValue::Error;
        }
    }
}

pub fn append_csv_operator() -> Operator {
    Operator::new(
        "Append CSV",
        OperatorKind::AppendCSV,
        DataValue::Table(DataFrame::empty()),
        DataValue::Table(DataFrame::empty()),
        OperatorCategory::IO,
        HashMap::from([
            ("another_csv_file_path".to_string(), PropertyValue::String("".to_string()))
        ])
    )
}

pub fn handle_append_csv_operator_execution(
    task_sender: &Sender<TaskChannelEvent>,
    input: &DataValue,
    properties: &HashMap<String, PropertyValue>
) -> DataValue {
    // Implementation goes here.
    //
    // Append the content of "another_csv_file_path" property to the dataframe of input.
    //
    // Example:
    // File 1 (input)          File 2                  -> Result
    //
    // |Names    |Ages |       |Names    |Ages  |         |Names    |Ages  |
    // |John     |28   |       |Alex     |38    |         |John     |28    |
    // |Richard  |32   |       |Henesy   |24    |         |Richard  |32    |
    //                                                    |Alex     |38    |
    //                                                    |Henesy   |24    |
    //
    // - PropertyValue definition:
    // pub enum PropertyValue {
    //     #[default]
    //     None,
    //     String(String),
    //     Int(i32),
    //     Float(f32),
    //     Bool(bool),
    //     List(Vec<PropertyValue>),
    //     Map(HashMap<String, PropertyValue>)
    // }
    //
    // - DataValue definition:
    // #[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
    // pub enum DataValue {
    //     #[default]
    //     None,
    //     Csv(String),
    //     FilePath(String),
    //     Table(DataFrame),
    //     Model,
    //     Error
    // }
    //
    // - If encouters any errors log error message and return DataValue::Error :
    // ```
    // let _ = task_sender.send(TaskChannelEvent::LogMessage(
    //     log_error("[Append CSV] {e}")
    // ));
    // return DataValue::Error;
    // ```
    //
    // - When execution is completed without errors, log normal message and return DataFrame
    // ```
    // let _ = task_sender.send(TaskChannelEvent::LogMessage(
    //     log_normal("[Append CSV] Appended CSV content")
    // ));
    // return DataValue::Table(df);
    // ```

    DataValue::None
}
