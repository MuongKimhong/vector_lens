use bevy::prelude::*;
use super::*;

pub fn replace_missing_value_operator() -> Operator {
    Operator::new(
        "Replace missing value",
        OperatorKind::ReplaceMissingValue,
        DataValue::Table(DataFrame::empty()),
        DataValue::Table(DataFrame::empty()),
        OperatorCategory::Cleaning,
        HashMap::from([
            ("replace_with".to_string(), PropertyValue::String("".to_string()))
        ])
    )
}

pub fn handle_replace_missing_value_operator_execution(
    task_sender: &Sender<TaskChannelEvent>,
    input: &DataValue,
    properties: &HashMap<String, PropertyValue>
) {

}
