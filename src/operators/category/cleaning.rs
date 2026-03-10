use bevy::prelude::*;
use super::*;

pub fn replace_missing_value_operator() -> Operator {
    Operator::new(
        "Replace missing value",
        OperatorKind::ReplaceMissingValue,
        DataValue::Table,
        DataValue::Table,
        OperatorCategory::Cleaning,
        HashMap::from([
            ("replace_with".to_string(), PropertyValue::String("".to_string()))
        ])
    )
}
