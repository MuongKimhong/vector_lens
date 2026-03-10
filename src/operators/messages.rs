use bevy::prelude::*;
use super::*;

pub enum ToggleOpContextState {
    Show,
    Hide
}

#[derive(Message)]
pub struct ToggleOpContext {
    pub state: ToggleOpContextState,
    pub op_box: Option<OpBox>,
    pub position: Vec2
}

impl ToggleOpContext {
    pub fn show(op_box: &OpBox, position: &Vec2) -> Self {
        Self {
            state: ToggleOpContextState::Show,
            op_box: Some(op_box.clone()),
            position: position.to_owned()
        }
    }

    pub fn hide() -> Self {
        Self {
            state: ToggleOpContextState::Hide,
            op_box: None,
            position: Vec2::default()
        }
    }
}
