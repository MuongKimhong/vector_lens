use bevy::prelude::*;
use bevy::tasks::{block_on, Task};
use bevy::tasks::futures_lite::future;
use crate::utils::create_log_with_timestamp;

use super::*;

// cubic spline rendering example https://bevy.org/examples/math/cubic-splines/

pub const OPERATOR_SIZE: Vec2 = Vec2::new(55.0, 55.0);

#[derive(Default, Debug, Clone)]
pub enum OpConnectButtonType {
    #[default]
    None,
    Input,
    Output
}

#[derive(Component, Clone)]
pub struct OpBox {
    pub id: Uuid,
    pub name: String,
}

impl OpBox {
    pub fn new(id: Uuid, name: &str) -> Self {
        Self {
            id,
            name: name.to_string()
        }
    }
}

#[derive(Component, Default, Debug)]
pub struct OpConnectButton {
    pub connected: bool,
    pub button_type: OpConnectButtonType
}

impl OpConnectButton {
    pub fn new_as_input() -> Self {
        Self {
            connected: false,
            button_type: OpConnectButtonType::Input
        }
    }

    pub fn new_as_output() -> Self {
        Self {
            connected: false,
            button_type: OpConnectButtonType::Output
        }
    }
}

#[derive(Component)]
pub struct OpConnectionLine {
    pub input_button_entity: Option<Entity>,
    pub output_button_entity: Option<Entity>
}

impl OpConnectionLine {
    pub fn new(input_entity: Option<Entity>, output_entity: Option<Entity>) -> Self {
        Self {
            input_button_entity: input_entity,
            output_button_entity: output_entity
        }
    }
}

pub fn handle_op_background_execution_system(
    mut commands: Commands,
    mut ui_state: ResMut<UiState>,
    mut processing_tasks: Query<&mut ProcessingTask>,
    mut operator_q: Query<(Entity, &OperatorNameEntity, &mut Operator)>,
    mut text_colors: Query<&mut TextColor>,
    mut console_log: ResMut<ConsoleLog>,
    sender: Option<Res<TaskChannelSender>>
) {
    let Some(executing_op_entity) = ui_state.executing_operator else {
        return;
    };

    let mut current_op_result = DataValue::None;
    let mut next_op_entity: Option<Entity> = None;

    {
        let Ok((entity, op_name_entity, op)) = operator_q.get_mut(executing_op_entity) else {
            return;
        };

        // currently, this is a task being executed
        if let Ok(mut task) = processing_tasks.get_mut(executing_op_entity) {
            let task_result = block_on(future::poll_once(&mut task.0));

            if let Some(result) = task_result {
                commands.entity(executing_op_entity).remove::<ProcessingTask>();

                if let Ok(mut color) = text_colors.get_mut(op_name_entity.0) {
                    *color = TextColor::default();
                }

                // current operator produce error result, stop the process
                if result == DataValue::Error {
                    ui_state.is_running = false;
                    ui_state.executing_operator = None;
                    return;
                }
                current_op_result = result;
                next_op_entity = op.next_operator;

                // point executing_operator to next op
                ui_state.executing_operator = op.next_operator;

                if op.next_operator.is_none() {
                    ui_state.is_running = false;

                    let log = create_log_with_timestamp("Finished process");
                    console_log.new_message(LogType::Success(log));
                    return;
                }
            }
        }

        // no background task, create one
        else if let Some(sender) = sender {
            let task = op.spawn_task(&sender.0);
            commands.entity(entity).insert(ProcessingTask(task));

            if let Ok(mut color) = text_colors.get_mut(op_name_entity.0) {
                color.0 = Color::srgb(1.0, 1.0, 0.0);
            }
        }
    }

    {
        if next_op_entity.is_some() && current_op_result != DataValue::None {
            let Ok((_, _, mut operator)) = operator_q.get_mut(next_op_entity.unwrap()) else {
                return;
            };

            operator.input = current_op_result;
        }
    }
}

pub fn handle_insert_task_channel_resource_system(
    mut commands: Commands,
    sender: Option<Res<TaskChannelSender>>,
    receiver: Option<Res<TaskChannelReceiver>>
) {
    if sender.is_none() && receiver.is_none() {
        let (sender, receiver) = unbounded::<TaskChannelEvent>();
        commands.insert_resource(TaskChannelSender(sender));
        commands.insert_resource(TaskChannelReceiver(receiver));
    }
}

pub fn listen_to_task_channel_receiver_system(
    receiver: Option<Res<TaskChannelReceiver>>,
    mut console_log: ResMut<ConsoleLog>
) {
    let Some(receiver) = receiver else {
        return;
    };

    while let Ok(channel_event) = receiver.0.try_recv() {
        match channel_event {
            TaskChannelEvent::LogMessage(log_type) => console_log.new_message(log_type),
            _ => {}
        }
    }
}
