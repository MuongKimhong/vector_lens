use bevy::prelude::*;
use makara::prelude::*;

use crate::resources::*;
use crate::operators::*;
use crate::canvas::*;

#[derive(Resource, Debug)]
pub struct OperatorPanelShowState(pub bool);

impl OperatorPanelShowState {
    pub fn new() -> Self {
        Self(true)
    }

    pub fn toggle(&mut self) {
        self.0 = !self.0;
    }

    pub fn state(&self) -> bool {
        self.0
    }
}

pub fn handle_show_and_hide_operator_panel(
    mut column_q: ColumnQuery,
    panel_state: Res<OperatorPanelShowState>,
) {
    if !panel_state.is_changed() {
        return;
    }

    if let Some(column) = column_q.find_by_id("operator-panel") {
        match panel_state.state() {
            true => column.style.node.display = Display::default(),
            false => column.style.node.display = Display::None,
        }
    }
}

pub fn operator_panel(operator_list: &OperatorList) -> impl Bundle {
    let operators = operator_list.0.clone();

    column_!(
        id: "operator-panel";

        [
            row_!(justify_content: JustifyContent::SpaceBetween; [
                text_!("Operators", font_size: 14.0),
                button_!("x", class: "is-light"; on: on_close_button_clicked)
            ]),

            text_input_!(
                "Search operator",
                width: percent(100),
                margin_top: px(10),
                on: on_search_input_change
            ),

            scroll_!(
                width: percent(100),
                height: percent(100),
                margin_top: px(10);

                iter [
                    operators.into_iter().map(|op| {
                        row_!(
                            insert: OperatorName(op.name.clone()),

                            [
                                button_!(op.name.as_str(), class: "is-light operator-btn"),
                                button_!(
                                    "+", class: "is-light", shadow: None, width: px(25);

                                    on: move |
                                        _: On<Clicked>,
                                        mut commands: Commands,
                                        mut meshes: ResMut<Assets<Mesh>>,
                                        mut materials: ResMut<Assets<ColorMaterial>>,
                                        mut operator_in_use: ResMut<OperatorInUseList>
                                    | {
                                        let mut new_op = Operator::new_from(&op);
                                        let entity = spawn_operator_entity(&mut commands, &mut meshes, &mut materials, &new_op);
                                        new_op.entity = Some(entity);

                                        operator_in_use.0.push(new_op);
                                    }
                                ),
                            ]
                        )
                    })
                ]
            )
        ]
    )
}

fn on_close_button_clicked(_: On<Clicked>, mut panel_state: ResMut<OperatorPanelShowState>) {
    panel_state.toggle();
}

fn on_search_input_change(
    change: On<Change<String>>,
    mut operator_names: Query<(&mut Node, &OperatorName)>
) {
    let search_text = change.data.to_lowercase();

    for (mut node, name) in operator_names.iter_mut() {
        if name.0.to_lowercase().contains(&search_text) || search_text.trim().is_empty() {
            node.display = Display::default();
        }
        else {
            node.display = Display::None;
        }
    }
}
