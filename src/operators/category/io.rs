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
    properties: &HashMap<String, PropertyValue>
) -> DataValue {
    println!("start read csv");
    let path = match properties.get("path") {
        Some(PropertyValue::String(s)) => s,
        _ => return DataValue::None,
    };

    let df = CsvReadOptions::default()
        .with_has_header(true)
        .with_infer_schema_length(None)
        .with_low_memory(true)
        .try_into_reader_with_file_path(Some(path.into()))
        .and_then(|reader| reader.finish());

    match df {
        Ok(frame) => {
            println!("Finish read csv {:?}", frame);
            return DataValue::Table(frame)
        },
        Err(e) => {
            eprintln!("CSV Read Error: {}", e);
            DataValue::None
        }
    }
}
