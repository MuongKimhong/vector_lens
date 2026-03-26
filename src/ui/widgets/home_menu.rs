use makara::prelude::*;
use bevy::prelude::*;
use bevy::app::AppExit;
use crossbeam_channel::{unbounded, Sender, Receiver};
use rfd::FileDialog;
use std::path::PathBuf;
use std::thread;

use crate::operators::{Operator, OpConnectButton};
use crate::resources::{ProcessFileState, OperatorInUseList, UiState};
use crate::{OperatorFormat, SaveProcessAsBackgroundThreadReceiver};

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

fn on_save_process_as_btn_clicked(
    _: On<Clicked>,
    mut save_as_thread_receiver: ResMut<SaveProcessAsBackgroundThreadReceiver>,
    operator_in_use: Res<OperatorInUseList>,
    ui_state: Res<UiState>,
    operators: Query<(&GlobalTransform, &Operator)>,
    // operator_btns: Query<(&GlobalTransform, &OpConnectButton)>
) {
    if operator_in_use.0.len() < 2 || ui_state.executing_operator.is_none() {
        return;
    }

    let (sender, receiver) = unbounded::<Option<PathBuf>>();
    save_as_thread_receiver.receiver = Some(receiver);

    let op_formats: Vec<OperatorFormat> = operators
        .iter()
        .map(|(transform, op)| OperatorFormat::new(transform.translation().truncate(), op))
        .collect();

    thread::spawn(move || {
        let dialog_result = FileDialog::new()
            .set_title("Export Process Configuration")
            .set_file_name("new_process.json")
            .add_filter("JSON File", &["json"])
            .save_file();

        let _ = sender.send(dialog_result);
    });
}

fn on_open_process_btn_clicked(_: On<Clicked>) {}

pub fn handle_save_process_as_thread_receive_result_system(
    mut receiver: ResMut<SaveProcessAsBackgroundThreadReceiver>
) {
    if let Some(receiver) = &receiver.receiver {
        if let Ok(result) = receiver.try_recv() {
            match result {
                Some(path) => {
                    // create json serialize of ProcessFormat here
                }
                _ => {}
            }
        }
    }
}
