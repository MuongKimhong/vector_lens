/// Anything related to Saving process and opening process file.

use serde::{Serialize, Deserialize};
use bevy::prelude::*;
use crossbeam_channel::{unbounded, Receiver};
use std::thread;
use std::path::PathBuf;

use crate::canvas::spawn_operator_entity;
use crate::operators::*;
use crate::resources::*;
use crate::utils::*;

pub struct IoPlugin;

impl Plugin for IoPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ProcessFileState::default());
        app.insert_resource(SaveProcessAsBackgroundThreadReceiver::default());
        app.insert_resource(OpenProcessBackgroundThreadReceiver::default());

        app.add_systems(
            Update,
            (
                handle_save_process_as_select_destination_receive_result_system,
                handle_save_process_thread_receiver_result_system,
                handle_open_process_choose_file_result_receiver,
                handle_open_process_parse_content_result_receiver
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
            Ok(_) => console_log.new_message(log_success("New process has been saved")),
            Err(_) => console_log.new_message(log_error("Failed to save new process"))
        }

        thread_receiver.save_result_receiver = None;
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
                for (entity, op) in operator_q.iter() {
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
