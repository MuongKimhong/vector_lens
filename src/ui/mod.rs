pub mod styles;
pub mod widgets;

pub use styles::*;
pub use widgets::*;

use bevy::prelude::*;
use makara::prelude::*;

use super::*;

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
    operator_q: Query<(Entity, &Operator)>,
) {
    ui_state.is_running = !ui_state.is_running;

    for (entity, op) in operator_q.iter() {
        if op.is_first_operator {
            ui_state.executing_operator = Some(entity);
            break;
        }
    }
}
