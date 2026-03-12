use bevy::prelude::*;
use bevy::tasks::{block_on, Task};
use bevy::tasks::futures_lite::future;
use makara::prelude::*;
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
    mut operator_q: Query<(Entity, &mut Operator)>,
) {
    let Some(executing_op_entity) = ui_state.executing_operator else {
        return;
    };


}
