use makara::prelude::*;
use bevy::prelude::*;
use bevy::app::AppExit;

use crate::resources::ProcessFileState;

use super::*;

pub fn home_menu() -> impl Bundle {
    row_!(
        width: percent(100),
        height: auto(),
        background_color: rgb(0.9, 0.9, 0.9),

        [
            dropdown_!("File", class: "menu-btn is-light", item_position: "left", [
                button_!("Open process", class: "is-light", on: on_open_process_btn_clicked),
                button_!("Save process", class: "is-light", on: on_save_process_btn_clicked),
                button_!("Save process as", class: "is-light", on: on_save_process_as_btn_clicked),
                button_!("Exit", class: "is-light", on: on_exit_btn_clicked)
            ]),
            dropdown_!("View", class: "menu-btn is-light", item_position: "left", [
                button_!("Show/Hide operator panel", class: "is-light"; on: show_hide_operator_panel),
                button_!("Show/Hide console panel", class: "is-light"; on: show_hide_console_panel)
            ])
        ]
    )
}

fn show_hide_operator_panel(_: On<Clicked>, mut panel_state: ResMut<OperatorPanelShowState>) {
    panel_state.toggle();
}

fn show_hide_console_panel(_: On<Clicked>, mut panel_state: ResMut<ConsolePanelShowState>) {
    panel_state.toggle();
}

fn on_exit_btn_clicked(_: On<Clicked>, mut exit: MessageWriter<AppExit>) {
    exit.write(AppExit::Success);
}

fn on_save_process_btn_clicked(
    _: On<Clicked>,
    mut process_state: ResMut<ProcessFileState>
) {
    if !process_state.editing_existing_process {
        return;
    }
}

fn on_save_process_as_btn_clicked(_: On<Clicked>) {}

fn on_open_process_btn_clicked(_: On<Clicked>) {}
