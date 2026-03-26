use bevy::prelude::*;
use makara::prelude::*;

use crate::{operators::Operator, PropertyValue};
use super::*;

fn on_read_csv_file_picker_result(
    change: On<Change<String>>,
    panel_state: Res<PropertyPanelShowState>,
    mut operator_q: Query<&mut Operator>
) {
    if let Some(op_entity) = panel_state.op_entity {
        if let Ok(mut op) = operator_q.get_mut(op_entity) {
            op.properties.insert(
                "file_path".to_string(),
                PropertyValue::String(change.data.clone())
            );
        }
    }
}

pub fn read_csv_property_container() -> impl Bundle {
    column_!(
        id: "read-csv-property-container",
        class: "property-container",
        display: Display::None,

        [
            text_!("File path"),
            file_picker_!(on: on_read_csv_file_picker_result),

            text_!("*Description", margin_top: px(20)),
            text_!("Read content of provided CSV file and turn into DataFrame.", font_size: 11.5),
        ]
    )
}

pub fn replace_missing_value_property_container() -> impl Bundle {
    column_!(
        id: "replace-missing-value-property-container",
        class: "property-container",
        display: Display::None,

        [
            text_!("Replace missing value property")
        ]
    )
}
