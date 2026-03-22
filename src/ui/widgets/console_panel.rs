use bevy::prelude::*;
use makara::prelude::*;

use crate::resources::*;

#[derive(Resource, Debug)]
pub struct ConsolePanelShowState(pub bool);

impl ConsolePanelShowState {
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

pub fn handle_show_and_hide_console_panel(
    mut column_q: ColumnQuery,
    panel_state: Res<ConsolePanelShowState>
) {
    if !panel_state.is_changed() {
        return;
    }

    if let Some(column) = column_q.find_by_id("console-panel") {
        match panel_state.state() {
            true => column.style.node.display = Display::default(),
            false => column.style.node.display = Display::None
        }
    }
}

pub fn console_panel() -> impl Bundle {
    column_!(
        id: "console-panel";

        [
            row_!(justify_content: JustifyContent::SpaceBetween; [
                text_!("Console", font_size: 14.0),
                button_!("x", class: "is-light"; on: on_close_button_clicked)
            ]),

            scroll_!(
                id: "console-messages-scroll",
                width: percent(100),
                height: percent(100),
                margin_top: px(10),

                on: |_: On<WidgetBuilt>, mut console_log: ResMut<ConsoleLog>| {
                    console_log.set_changed();
                }
            )
        ]
    )
}

fn on_close_button_clicked(_: On<Clicked>, mut panel_state: ResMut<ConsolePanelShowState>) {
    panel_state.toggle();
}

pub fn detect_new_console_messages_system(
    console_log: Res<ConsoleLog>,
    mut scroll_q: ScrollQuery
) {
    if !console_log.is_changed() {
        return;
    }

    let Some(messages) = console_log.messages.get(&console_log.last_key_count) else {
        return;
    };

    let Some(last_message) = messages.last() else {
        return;
    };

    if let Some(mut scroll) = scroll_q.find_by_id("console-messages-scroll") {
        let (color, message) = match last_message {
            LogType::Error(msg) => (Color::srgb(1.0, 0.0, 0.0), msg),
            LogType::Success(msg) => (Color::srgb(0.09, 0.7, 0.35), msg),
            LogType::Normal(msg) => (Color::srgb(0.1, 0.1, 0.1), msg)
        };

        scroll.add_child(text_!(message, color: color));
    }
}
