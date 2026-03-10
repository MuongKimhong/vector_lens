use makara::prelude::*;
use bevy::prelude::*;
use bevy::app::AppExit;

use super::*;

pub fn home_menu() -> impl Bundle {
    row_!(
        width: percent(100),
        height: auto();

        [
            dropdown_!("File", class: "menu-btn", item_position: "left"; [
                button_!("New process", class: "is-light"),
                button_!("Open process", class: "is-light"),
                button_!("Save process", class: "is-light"),
                button_!("Save process as", class: "is-light"),
                button_!("Exit", class: "is-light"; on: |_clicked: On<Clicked>, mut exit: MessageWriter<AppExit>| {
                    exit.write(AppExit::Success);
                })
            ]),
            dropdown_!("View", class: "menu-btn", item_position: "left"; [
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
