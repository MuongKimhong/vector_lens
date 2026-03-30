/// Anything related to Saving process and opening process file.

use serde::{Serialize, Deserialize};
use bevy::prelude::*;
use crossbeam_channel::{unbounded, Receiver};
use std::thread;
use std::path::PathBuf;

use crate::operators::*;
use crate::resources::*;
use crate::utils::*;

pub struct IoPlugin;

impl Plugin for IoPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ProcessFileState::default());
        app.insert_resource(SaveProcessAsBackgroundThreadReceiver::default());

        app.add_systems(
            Update,
            (
                handle_save_process_as_select_destination_receive_result_system,
                handle_save_process_thread_receiver_result_system
            )
        );
    }
}

/// Resource used to keep track of process file.
/// - Is user editing an existing process?
/// - Is user using application without any process file?
#[derive(Resource, Debug, Default)]
pub struct ProcessFileState {
    pub editing_existing_process: bool,
    pub currernt_process_path: Option<PathBuf>,
    pub file_needs_to_be_saved: bool
}

/// A resource used to store the path of selected destination
/// when user want to save new process (Save process as).
#[derive(Resource, Debug, Default)]
pub struct SaveProcessAsBackgroundThreadReceiver {
    pub destination_receiver: Option<Receiver<Option<PathBuf>>>,
    pub save_result_receiver: Option<Receiver<std::io::Result<()>>>,
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

#[derive(Deserialize, Serialize)]
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
