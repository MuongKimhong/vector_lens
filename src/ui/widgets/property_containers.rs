use bevy::prelude::*;
use makara::prelude::*;
use crossbeam_channel::{unbounded, Receiver};
use rfd::FileDialog;
use std::path::PathBuf;

use crate::{operators::Operator, PropertyValue};
use crate::resources::*;
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

fn on_read_excel_file_picker_result(
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

pub fn read_excel_property_container() -> impl Bundle {
    column_!(
        id: "read-excel-property-container",
        class: "property-container",
        display: Display::None,

        [
            text_!("File path"),
            file_picker_!(on: on_read_excel_file_picker_result),

            text_!("*Description", margin_top: px(20)),
            text_!("Read content of provided Excel file and turn into DataFrame.", font_size: 11.5),
        ]
    )
}

fn on_file_name_input_change(
    change: On<Change<String>>,
    panel_state: Res<PropertyPanelShowState>,
    mut operator_q: Query<&mut Operator>
) {
    if let Some(op_entity) = panel_state.op_entity {
        if let Ok(mut op) = operator_q.get_mut(op_entity) {
            op.properties.insert(
                "file_name".to_string(),
                PropertyValue::String(change.data.clone())
            );
        }
    }
}

fn on_file_type_selector_change(
    change: On<Change<String>>,
    panel_state: Res<PropertyPanelShowState>,
    mut operator_q: Query<&mut Operator>
) {
    if let Some(op_entity) = panel_state.op_entity {
        if let Ok(mut op) = operator_q.get_mut(op_entity) {
            op.properties.insert(
                "file_type".to_string(),
                PropertyValue::String(change.data.clone())
            );
        }
    }
}

fn on_select_destination_btn_clicked(
    _: On<Clicked>,
    mut thread_receiver: ResMut<SaveCsvOrExcelBackgroundThreadReceiver>
) {
    let (sender, receiver) = unbounded::<Option<PathBuf>>();
    thread_receiver.destination_receiver = Some(receiver);

    std::thread::spawn(move || {
        let dialog_result = FileDialog::new()
            .set_title("Save to CSV or Excel")
            .pick_folder();

        let _ = sender.send(dialog_result);
    });
}

pub fn save_to_csv_or_excel_property_container() -> impl Bundle {
    column_!(
        id: "save-to-csv-or-excel-property-container",
        class: "property-container",
        display: Display::None,

        [
            text_!("Save Destionation:"),
            text_!("", id: "save-destination-display"),
            button_!(
                "Select Destination",
                id: "select-destination-btn",
                on: on_select_destination_btn_clicked
            ),

            text_!("File name", margin_top: px(20)),
            text_input_!(
                "Enter file name without extension",
                margin_top: px(10),
                width: percent(100),
                on: on_file_name_input_change
            ),

            text_!("File type", margin_top: px(20)),
            select_!(
                "Select file type",
                choices: &["Csv", "Excel"],
                margin_top: px(10),
                on: on_file_type_selector_change
            ),

            text_!("*Description", margin_top: px(20)),
            text_!("Save content to CSV or Excel file.", font_size: 11.5),
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
