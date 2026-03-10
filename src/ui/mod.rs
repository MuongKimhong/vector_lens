pub mod styles;
pub mod widgets;

pub use styles::*;
pub use widgets::*;

use bevy::prelude::*;
use makara::prelude::*;

use super::*;

#[derive(Default, Debug)]
pub enum ViewingTab {
    #[default]
    Design,
    Result
}

#[derive(Resource, Debug, Default)]
pub struct UiState {
    pub viewing_tab: ViewingTab,
    pub is_running: bool
}

impl UiState {
    pub fn on_running_flag_change_system(state: Res<UiState>, mut btn_q: ButtonQuery) {
        if !state.is_changed() { return; }

        if let Some(mut btn) = btn_q.find_by_id("run-btn") {
            if state.is_running {
                btn.set_text("Stop");
                btn.class.set_class("is-danger");
            }
            else {
                btn.set_text("Run");
                btn.class.set_class("is-success");
            }
        }
    }

    pub fn on_viewing_tab_change_system(state: Res<UiState>, mut btn_q: ButtonQuery) {
        if !state.is_changed() { return; }

        let (design_class, result_class) = match state.viewing_tab {
            ViewingTab::Design => ("is-primary-dark", "is-light"),
            ViewingTab::Result => ("is-light", "is-primary-dark"),
        };

        if let Some(btn) = btn_q.find_by_id("design-tab-btn") {
            btn.class.set_class(design_class);
        }

        if let Some(btn) = btn_q.find_by_id("result-tab-btn") {
            btn.class.set_class(result_class);
        }
    }
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MakaraPlugin::default());

        app.insert_resource(UiState::default());
        app.insert_resource(OperatorPanelShowState::new());
        app.insert_resource(ConsolePanelShowState::new());
        app.insert_resource(PropertyPanelShowState::new());
        app.insert_resource(RightClickedOperatorBox(None));

        app.add_systems(
            Startup,
            (setup_styles, setup_home_view)
        );

        app.add_systems(Update, (
            handle_show_and_hide_operator_panel,
            handle_show_and_hide_console_panel,
            handle_show_and_hide_property_panel,
            handle_show_and_hide_op_context_menu_system
        ));

        app.add_systems(Update, (
            UiState::on_running_flag_change_system,
            UiState::on_viewing_tab_change_system
        ));
    }
}

pub fn setup_home_view(
    mut commands: Commands,
    operator_list: Res<OperatorList>
) {
    commands.spawn(
        root_!(
            route: "home",
            background_color: "transparent";

            [
                home_menu(),
                row_!(
                    width: percent(100),
                    height: percent(100),
                    margin_top: px(10),
                    align_items: AlignItems::Start;

                    [
                        operator_panel(&operator_list),
                        console_panel(),
                        property_panel(),
                        operator_context(),

                        row_!(
                            position_type: PositionType::Absolute,
                            width: auto(),
                            left: percent(50),
                            height: auto();

                            [
                                button_!(
                                    "Design",
                                    id: "design-tab-btn",
                                    class: "is-primary-dark",
                                    shadow: None,
                                    border_radius_from: (px(5), px(0), px(0), px(5));
                                    on: on_design_btn_clicked
                                ),
                                button_!(
                                    "Result",
                                    id: "result-tab-btn",
                                    class: "is-light",
                                    shadow: None,
                                    border_radius: px(0);
                                    on: on_result_btn_clicked
                                ),
                                button_!(
                                    "Run",
                                    id: "run-btn",
                                    class: "is-success",
                                    shadow: None,
                                    border_radius_from: (px(0), px(5), px(5), px(0));
                                    on: on_run_btn_clicked
                                )
                            ]
                        )
                    ]
                )
            ]
        )
    );
}

fn on_design_btn_clicked(_: On<Clicked>, mut ui_state: ResMut<UiState>) {
    ui_state.viewing_tab = ViewingTab::Design;
}

fn on_result_btn_clicked(_: On<Clicked>, mut ui_state: ResMut<UiState>) {
    ui_state.viewing_tab = ViewingTab::Result;
}

fn on_run_btn_clicked(
    _: On<Clicked>,
    mut ui_state: ResMut<UiState>,
    mut operator_q: Query<(Entity, &mut Operator)>,
) {
    ui_state.is_running = !ui_state.is_running;

    let mut first_op_entity: Option<Entity> = None;
    let mut exhausted = false;

    for (entity, op) in operator_q.iter() {
        if op.is_first_operator {
            first_op_entity = Some(entity);
            break;
        }
    }

    if let Some(first_op_entity) = first_op_entity {
        let mut current_op_entity = first_op_entity;

        while !exhausted {
            if let Ok((_, mut op)) = operator_q.get_mut(current_op_entity) {
                op.execute();

                if let Some(next_op) = op.next_operator {
                    current_op_entity = next_op;
                }
                else {
                    break;
                }
            }
        }
    }
}
