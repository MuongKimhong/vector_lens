use bevy::prelude::*;
use makara::prelude::*;
use crossbeam_channel::{unbounded, Receiver};
use polars::prelude::{CsvReadOptions, CsvWriter, DataFrame, SerReader, SerWriter};
use rfd::FileDialog;
use std::path::PathBuf;

use crate::{operators::Operator, PropertyValue, DataValue};
use crate::resources::*;
use super::*;

fn on_read_csv_file_picker_result(
    change: On<Change<String>>,
    panel_state: Res<PropertyPanelShowState>,
    mut operator_q: Query<&mut Operator>,
    mut thread_receiver: ResMut<PreReadCsvOrExcelContentThreadReceiver>
) {
    if let Some(op_entity) = panel_state.op_entity {
        if let Ok(mut op) = operator_q.get_mut(op_entity) {
            op.properties.insert(
                "file_path".to_string(),
                PropertyValue::String(change.data.clone())
            );

            if !change.data.is_empty() {
                let (sender, receiver) = unbounded::<DataValue>();
                let path = change.data.clone();
                thread_receiver.0 = Some(receiver);

                std::thread::spawn(move || {
                    let df = CsvReadOptions::default()
                        .with_has_header(true)
                        .with_infer_schema_length(None)
                        .with_low_memory(true)
                        .try_into_reader_with_file_path(Some(PathBuf::from(&path)))
                        .and_then(|reader| reader.finish());

                    if let Ok(frame) = df {
                        let _ = sender.send(DataValue::Table(frame));
                    }
                });
            }
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
            text_!("Columns with missing values:"),
            column_!(
                id: "missing-value-columns-wrapper",
                margin_top: px(10),
                []
            ),

            column_!(id: "missing-value-strategies-wrapper", margin_top: px(20), display: Display::None, [
                text_!("*Null: Leave it as empty or as it is.", class: "strategy-text"),
                text_!("*Mean: Replaces with the column's average (numeric only)", class: "strategy-text"),
                text_!("*Max: Replaces with the highest value in the column (numeric only)", class: "strategy-text"),
                text_!("*Min: Replaces with the lowest value in the column", class: "strategy-text"),
                text_!("*Mode: Replaces with the most frequent value (best for categories)", class: "strategy-text"),
                text_!("*Foward Fill: Carries the last valid value forward to fill the next gap", class: "strategy-text"),
                text_!("*Backward Fill: Reaches ahead to the next valid value and pulls it back", class: "strategy-text"),
            ]),

            text_!("*Description", margin_top: px(20)),
            text_!(
                "Replace missing value on each column with given strategies such as null, mean, max, min, mode, foward fill, backward fill and unknown.",
                font_size: 11.5
            ),
        ]
    )
}
