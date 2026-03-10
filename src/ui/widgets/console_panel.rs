use bevy::prelude::*;
use makara::prelude::*;

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
                width: percent(100),
                height: percent(100),
                margin_top: px(10);

                [
                    text_!("This is some text"),
                    text_!("This is some text"),
                    text_!("This is some text"),
                ]
            )
        ]
    )
}

fn on_close_button_clicked(_: On<Clicked>, mut panel_state: ResMut<ConsolePanelShowState>) {
    panel_state.toggle();
}
