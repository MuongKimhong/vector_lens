/// Anything related to Saving process and opening process file.

use serde::{Serialize, Deserialize};
use makara::prelude::*;
use bevy::prelude::*;
use bevy::input::{ButtonInput, keyboard::KeyCode};
use crossbeam_channel::unbounded;
use std::collections::HashMap;
use std::thread;

use crate::canvas::spawn_operator_entity;
use crate::ui::widgets::PropertyPanelShowState;
use crate::messages::*;
use crate::operators::*;
use crate::resources::*;
use crate::utils::*;

pub struct IoPlugin;

impl Plugin for IoPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ProcessFileState::default());
        app.insert_resource(SaveProcessAsBackgroundThreadReceiver::default());
        app.insert_resource(OpenProcessBackgroundThreadReceiver::default());
        app.insert_resource(SaveCsvOrExcelBackgroundThreadReceiver::default());
        app.insert_resource(PreReadCsvOrExcelContentThreadReceiver::default());

        app.add_message::<SaveProcess>();

        app.add_systems(
            Update,
            (
                handle_save_process_as_select_destination_receive_result_system,
                handle_save_process_thread_receiver_result_system,
                handle_open_process_choose_file_result_receiver,
                handle_open_process_parse_content_result_receiver,
                handle_save_to_csv_or_excel_select_destination_receive_result_system,
                handle_save_process,
                detect_ctrl_s_pressed_to_save_process,
                pre_read_csv_content
            )
        );
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OperatorFormat {
    pub transform_x: f32,
    pub transform_y: f32,
    pub op_object: Operator
}

impl OperatorFormat {
    pub fn new(translation: Vec2, op: &Operator) -> Self {
        Self {
            transform_x: translation.x,
            transform_y: translation.y,
            op_object: op.clone()
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ProcessFileFormat {
    pub operators: Vec<OperatorFormat>,
    pub file_name: String
}

impl ProcessFileFormat {
    pub fn new(operators: Vec<OperatorFormat>, file_name: String) -> Self {
        Self {
            operators,
            file_name
        }
    }
}

pub fn handle_save_process_as_select_destination_receive_result_system(
    mut thread_receiver: ResMut<SaveProcessAsBackgroundThreadReceiver>,
    operator_q: Query<(&GlobalTransform, &Operator)>
) {
    let Some(receiver) = &thread_receiver.destination_receiver else {
        return;
    };

    if let Ok(result) = receiver.try_recv() {
        match result {
            Some(path) => {
                let op_formats = operator_q
                    .iter()
                    .map(|(transform, op)| {
                        let translation = transform.translation().truncate();
                        OperatorFormat::new(translation, op)
                    })
                    .collect::<Vec<OperatorFormat>>();

                let process_file_format = ProcessFileFormat::new(op_formats, path.display().to_string());
                let data = serde_json::to_string(&process_file_format);

                if let Ok(data) = data {
                    let (sender, receiver) = unbounded::<std::io::Result<()>>();
                    thread_receiver.save_result_receiver = Some(receiver);

                    thread::spawn(move || {
                        let _ = sender.send(write_file(path, data.as_bytes()));
                    });
                }
            }
            _ => {}
        }

        // user finished selecting destionation or cancelled, no longer need receiver
        thread_receiver.destination_receiver = None;
    }
}

pub fn handle_save_process_thread_receiver_result_system(
    mut thread_receiver: ResMut<SaveProcessAsBackgroundThreadReceiver>,
    mut console_log: ResMut<ConsoleLog>
) {
    let Some(receiver) = &thread_receiver.save_result_receiver else {
        return;
    };

    if let Ok(result) = receiver.try_recv() {
        match result {
            Ok(_) => console_log.new_message(log_success("Process has been saved")),
            Err(_) => console_log.new_message(log_error("Failed to save new process"))
        }

        thread_receiver.save_result_receiver = None;
    }
}

pub fn handle_save_to_csv_or_excel_select_destination_receive_result_system(
    mut thread_receiver: ResMut<SaveCsvOrExcelBackgroundThreadReceiver>,
    mut operator_q: Query<&mut Operator>,
    panel_state: Res<PropertyPanelShowState>,
) {
    let Some(receiver) = &thread_receiver.destination_receiver else {
        return;
    };

    if let Ok(result) = receiver.try_recv() {
        match result {
            Some(path) => if let Some(op_entity) = panel_state.op_entity {
                if let Ok(mut op) = operator_q.get_mut(op_entity) {
                    op.properties.insert(
                        "file_path".to_string(),
                        PropertyValue::String(path.display().to_string())
                    );
                }
            }
            _ => {}
        }

        // user finished selecting destionation or cancelled, no longer need receiver
        thread_receiver.destination_receiver = None;
    }
}

pub fn handle_open_process_choose_file_result_receiver(
    mut thread_receiver: ResMut<OpenProcessBackgroundThreadReceiver>,
    mut process_file_state: ResMut<ProcessFileState>,
    mut console_log: ResMut<ConsoleLog>
) {
    let Some(receiver) = &thread_receiver.choose_file_receiver else {
        return;
    };

    if let Ok(result) = receiver.try_recv() {
        match result {
            Some(path) => {
                console_log.new_message(log_normal("Opening process"));

                let (sender, receiver) = unbounded::<Result<ProcessFileFormat, serde_json::Error>>();
                thread_receiver.open_result_receiver = Some(receiver);

                process_file_state.currernt_process_path = Some(path.clone());

                thread::spawn(move || {
                    // let _ = sender.send()
                    if let Ok(content) = std::fs::read_to_string(path) {
                        let process_format = serde_json::from_str::<ProcessFileFormat>(&content);
                        let _ = sender.send(process_format);
                    }
                });
            }
            _ => console_log.new_message(log_error("Failed to open process file"))
        }
        thread_receiver.choose_file_receiver = None;
    }
}

pub fn handle_open_process_parse_content_result_receiver(
    mut thread_receiver: ResMut<OpenProcessBackgroundThreadReceiver>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut op_in_use: ResMut<OperatorInUseList>,
    mut connected_curves: ResMut<ConnectedCurves>,
    mut msg_writer: MessageWriter<ConstructConnectedCurvesAfterOpenProcess>,
    mut process_file_state: ResMut<ProcessFileState>,
    mut console_log: ResMut<ConsoleLog>,
    operator_q: Query<(Entity, &Operator)>,
) {
    let Some(receiver) = &thread_receiver.open_result_receiver else {
        return;
    };

    if let Ok(result) = receiver.try_recv() {
        match result {
            Ok(content) => {
                // despawn any existing operators
                for (entity, _op) in operator_q.iter() {
                    commands.entity(entity).despawn();
                }

                // empty operator in use resource and connected curves
                op_in_use.0 = Vec::new();
                connected_curves.0 = Vec::new();

                // spawn entity at exact translation
                for op in content.operators.iter() {
                    let mut new_op = op.op_object.clone();
                    let op_entity = spawn_operator_entity(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &op.op_object,
                        Some(Vec2::new(op.transform_x, op.transform_y))
                    );
                    new_op.entity = Some(op_entity);
                    op_in_use.0.push(new_op);
                }

                // send event to construct connected curves
                msg_writer.write(ConstructConnectedCurvesAfterOpenProcess);

                process_file_state.editing_existing_process = true;
            }
            Err(e) => {
                process_file_state.reset();
                console_log.new_message(log_error(&format!("{e}")));
            }
        }
        thread_receiver.open_result_receiver = None;
    }
}

pub fn handle_save_process(
    mut messages: MessageReader<SaveProcess>,
    mut thread_receiver: ResMut<SaveProcessAsBackgroundThreadReceiver>,
    process_state: Res<ProcessFileState>,
    operator_q: Query<(&GlobalTransform, &Operator)>
) {
    for _ in messages.read() {
        if !process_state.can_save() {
            continue;
        }

        if let Some(path) = &process_state.currernt_process_path {
            let path = path.clone();
            let op_formats = operator_q
                .iter()
                .map(|(transform, op)| {
                    let translation = transform.translation().truncate();
                    OperatorFormat::new(translation, op)
                })
                .collect::<Vec<OperatorFormat>>();

            let process_file_format = ProcessFileFormat::new(op_formats, path.display().to_string());
            let data = serde_json::to_string(&process_file_format);

            if let Ok(data) = data {
                let (sender, receiver) = unbounded::<std::io::Result<()>>();
                thread_receiver.save_result_receiver = Some(receiver);

                thread::spawn(move || {
                    let _ = sender.send(write_file(path, data.as_bytes()));
                });
            }
        }
    }
}

pub fn detect_ctrl_s_pressed_to_save_process(
    keys: Res<ButtonInput<KeyCode>>,
    mut console_log: ResMut<ConsoleLog>,
    mut msg: MessageWriter<SaveProcess>
) {
    if is_control_key_held(&keys) && keys.just_pressed(KeyCode::KeyS) {
        console_log.new_message(log_normal("Saving process"));
        msg.write(SaveProcess);
    }
}

fn on_missing_value_replace_with_selector_change(
    change: On<Change<String>>,
    mut operator_q: Query<&mut Operator>,
    mut select_q: SelectQuery
) {
    if let Some(selector) = select_q.find_by_entity(change.entity) {
        let class = selector.class.value.clone();
        let Some(column_name) = class.split("-").last() else {
            return;
        };

        for mut op in operator_q.iter_mut() {
            if op.kind != OperatorKind::ReplaceMissingValue {
                continue;
            }

            let Some(property) = op.properties.get_mut("columns_with_missing_value") else {
                return;
            };

            match property {
                PropertyValue::Map(map) => {
                    map.insert(column_name.to_string(), PropertyValue::String(change.data.clone()));
                }
                _ => {}
            }
        }
    }
}

// rmv = replace missing value
pub fn handle_update_rmv_property_on_get_pre_read_content(
    dirty_columns: &Vec<String>,
    operator_q: &mut Query<&mut Operator>,
    column_q: &mut ColumnQuery,
    commands: &mut Commands
) {
    for mut op in operator_q.iter_mut() {
        let Some(property) = op.properties.get_mut("columns_with_missing_value") else {
            continue;
        };
        let mut map = HashMap::new();

        let Some(column_widget) = column_q.find_by_id("missing-value-columns-wrapper") else {
            return;
        };
        commands.entity(column_widget.entity).despawn_children();
        commands.entity(column_widget.entity).with_children(|parent| {
            for column in dirty_columns.iter() {
                map.insert(column.clone(), PropertyValue::String("".to_string()));

                parent.spawn(
                    column_!(
                        padding_top: px(6),
                        padding_bottom: px(6),

                        [
                            text_!(column, id: &format!("missing-value-name-{column}")),
                            select_!(
                                "Replace with",
                                choices: &[
                                    "Null",
                                    "Mean",
                                    "Max",
                                    "Min",
                                    "Mode",
                                    "Average",
                                    "Forward Fill",
                                    "Backward Fill",
                                    "Unknown"
                                ],
                                margin_top: px(5),
                                class: &format!("missing-value-replace-with-{column}"),
                                on: on_missing_value_replace_with_selector_change
                            )
                        ]
                    )
                );
            }
        });

        if let Some(strategy_wrapper_column) = column_q.find_by_id("missing-value-strategies-wrapper") {
            strategy_wrapper_column.style.node.display = Display::default();
        }

        *property = PropertyValue::Map(map);
    }
}

pub fn pre_read_csv_content(
    mut thread_receiver: ResMut<PreReadCsvOrExcelContentThreadReceiver>,
    mut pre_read_content: ResMut<PreReadCsvOrExcelContent>,
    mut update_rmv_property_msg_writer: MessageWriter<UpdateReplaceMissingValuePropertyAfterOPSpawned>,
    mut update_sa_property_msg_writer: MessageWriter<UpdateSelectAttributesPropertyAfterOPSpawned>
) {
    let Some(receiver) = &thread_receiver.get() else {
        return;
    };

    if let Ok(result) = receiver.try_recv() {
        if result == DataValue::None {
            return;
        }

        match &result {
            DataValue::Table(_df) => {
                pre_read_content.0 = result;
                thread_receiver.0 = None;
                update_rmv_property_msg_writer.write(UpdateReplaceMissingValuePropertyAfterOPSpawned);
                update_sa_property_msg_writer.write(UpdateSelectAttributesPropertyAfterOPSpawned);
            }
            _ => {}
        }
    }
}
