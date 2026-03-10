use super::*;

pub fn read_csv_operator() -> Operator {
    Operator::new(
        "Read CSV",
        OperatorKind::ReadCSV,
        DataValue::None,
        DataValue::Table,
        OperatorCategory::IO,
        HashMap::from([
            ("file_path".to_string(), PropertyValue::String("".to_string()))
        ])
    )
}
