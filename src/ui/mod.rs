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
        app.insert_resource(ConsoleLog::default());

        app.add_systems(
            Startup,
            (setup_styles, setup_home_view)
        );

        app.add_systems(Update, (
            handle_show_and_hide_operator_panel,
            handle_show_and_hide_console_panel,
            handle_show_and_hide_property_panel,
            handle_update_property_panel_content,
            handle_show_and_hide_op_context_menu_system,
            detect_new_console_messages_system
        ));

        app.add_systems(Update, (
            UiState::on_running_flag_change_system,
            UiState::on_viewing_tab_change_system
        ));
    }
}

/// Hold the entity of the current running operator.
#[derive(Component)]
pub struct OperatorRunningCircular(pub Entity);

pub fn setup_home_view(
    mut commands: Commands,
    operator_list: Res<OperatorList>
) {
    commands.spawn(
        root_!(
            route: "home",
            background_color: "transparent",

            on: |mut clicked: On<Pointer<Click>>, mut messages: MessageWriter<ToggleOpContext>| {
                messages.write(ToggleOpContext::hide());
                clicked.propagate(false);
            },

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

fn handle_stop_running_process(
    ui_state: &mut UiState,
    console_log: &mut ConsoleLog,
    commands: &mut Commands,
    text_colors: &mut Query<&mut TextColor>,
    operator_q: &Query<(Entity, &Operator, &OperatorNameEntity)>,
    processing_tasks: &Query<(Entity, &ProcessingTask)>
) {
    if let Some(running_entity) = ui_state.executing_operator {
        if let Ok((_, _, name_entity)) = operator_q.get(running_entity) {
            if let Ok(mut color) = text_colors.get_mut(name_entity.0) {
                *color = TextColor::default();
            }
        }
    }

    ui_state.is_running = false;
    ui_state.executing_operator = None;

    for (entity, _task) in processing_tasks.iter() {
        commands.entity(entity).remove::<ProcessingTask>();
    }

    let log = create_log_with_timestamp("Stopped process");
    console_log.new_message(LogType::Error(log));
}

fn handle_start_running_process(
    ui_state: &mut UiState,
    console_log: &mut ConsoleLog,
    operator_q: &Query<(Entity, &Operator, &OperatorNameEntity)>
) {
    for (entity, op, _) in operator_q.iter() {
        if op.is_first_operator {
            ui_state.executing_operator = Some(entity);
            ui_state.is_running = true;

            let log = create_log_with_timestamp("Started process");
            console_log.new_session();
            console_log.new_message(LogType::Success(log));
            break;
        }
    }

    if ui_state.executing_operator.is_none() {
        let log = create_log_with_timestamp("You need connected operators to start the process");
        console_log.new_message(LogType::Error(log));
    }
}

fn on_run_btn_clicked(
    _: On<Clicked>,
    mut commands: Commands,
    mut ui_state: ResMut<UiState>,
    mut console_log: ResMut<ConsoleLog>,
    mut text_colors: Query<&mut TextColor>,
    processing_tasks: Query<(Entity, &ProcessingTask)>,
    operator_q: Query<(Entity, &Operator, &OperatorNameEntity)>,
) {
    match ui_state.is_running {
        true => handle_stop_running_process(
            &mut ui_state,
            &mut console_log,
            &mut commands,
            &mut text_colors,
            &operator_q,
            &processing_tasks
        ),
        false => handle_start_running_process(
            &mut ui_state,
            &mut console_log,
            &operator_q
        )
    }

}
