// context menu, showed wwhen right clicked on active operator

use bevy::prelude::*;
use makara::prelude::*;
use uuid::Uuid;

use crate::{operators::messages::*, OpBox, OperatorInUseList};
use super::*;

/// Component use to store id of operator so that when deleting,
/// we know which operator to be deleted.
#[derive(Resource, Debug)]
pub struct RightClickedOperatorBox(pub Option<Uuid>);

pub fn operator_context() -> impl Bundle {
    column_!(
        id: "operator-context-menu",
        position_type: PositionType::Absolute,
        display: Display::None,
        width: px(100),
        height: auto();

        [
            button_!(
                "Properties",
                width: percent(100),
                border_radius_from: (px(5), px(5), px(0), px(0)),
                class: "is-light",
                on: on_property_btn_clicked
            ),
            button_!(
                "Delete",
                width: percent(100),
                border_radius_from: (px(0), px(0), px(5), px(5)),
                class: "is-light",
                on: on_delete_btn_clicked
            )
        ]
    )
}

pub fn handle_show_and_hide_op_context_menu_system(
    mut messages: MessageReader<ToggleOpContext>,
    mut column_q: ColumnQuery
) {
    for msg in messages.read() {
        if let Some(column) = column_q.find_by_id("operator-context-menu") {
            match msg.state {
                ToggleOpContextState::Show => {
                    column.style.node.display = Display::default();
                    column.style.node.left = px(msg.position.x);
                    column.style.node.top = px(msg.position.y);
                }
                ToggleOpContextState::Hide => {
                    column.style.node.display = Display::None;
                }
            }
        }
    }
}

fn on_property_btn_clicked(
    _: On<Clicked>,
    mut panel_state: ResMut<PropertyPanelShowState>,
    mut messages: MessageWriter<ToggleOpContext>,
    right_clicked_op_box: Res<RightClickedOperatorBox>,
    operator_q: Query<(Entity, &OpBox)>
) {
    if let Some(op_id) = right_clicked_op_box.0 {
        for (entity, op_box) in operator_q.iter() {
            if op_id != op_box.id {
                continue;
            }

            messages.write(ToggleOpContext::hide());

            panel_state.show();
            panel_state.op_id = Some(op_id);
            panel_state.op_entity = Some(entity);
            panel_state.property_type = match op_box.name.as_str() {
                "Read CSV" => PropertyType::ReadCsv,
                "Replace missing value" => PropertyType::ReplaceMissingValue,
                _ => PropertyType::None
            };
        }
    }
}

fn on_delete_btn_clicked(
    _: On<Clicked>,
    mut commands: Commands,
    mut panel_state: ResMut<PropertyPanelShowState>,
    mut messages: MessageWriter<ToggleOpContext>,
    mut right_clicked_op_box: ResMut<RightClickedOperatorBox>,
    mut op_in_use: ResMut<OperatorInUseList>,
    operator_q: Query<(Entity, &OpBox)>
) {
    if let Some(op_id) = right_clicked_op_box.0 {
        for (entity, op_box) in operator_q.iter() {
            if op_id != op_box.id {
                continue;
            }

            commands.entity(entity).despawn();
            panel_state.hide();
            messages.write(ToggleOpContext::hide());
            right_clicked_op_box.0 = None;

            for (i, op) in op_in_use.0.iter().enumerate() {
                if op.id == op_id {
                    op_in_use.0.remove(i);
                    break;
                }
            }
        }
    }
}
