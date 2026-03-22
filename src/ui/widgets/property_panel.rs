use bevy::prelude::*;
use makara::prelude::*;
use uuid::Uuid;

use super::*;

#[derive(Debug)]
pub enum PropertyType {
    None,
    ReadCsv,
    ReplaceMissingValue
}

#[derive(Resource, Debug)]
pub struct PropertyPanelShowState {
    pub state: bool,
    pub op_id: Option<Uuid>,
    pub op_entity: Option<Entity>,
    pub property_type: PropertyType
}

impl PropertyPanelShowState {
    pub fn new() -> Self {
        Self {
            state: false,
            op_id: None,
            op_entity: None,
            property_type: PropertyType::None
        }
    }

    pub fn reset(&mut self) {
        self.state = false;
        self.op_id = None;
        self.op_entity = None;
        self.property_type = PropertyType::None;
    }

    pub fn toggle(&mut self) {
        self.state = !self.state;
    }

    pub fn show(&mut self) {
        self.state = true;
    }

    pub fn hide(&mut self) {
        self.state = false;
    }

    pub fn state(&self) -> bool {
        self.state
    }
}

pub fn handle_show_and_hide_property_panel(
    mut column_q: ColumnQuery,
    mut panel_state: ResMut<PropertyPanelShowState>,
) {
    if !panel_state.is_changed() {
        return;
    }

    if let Some(column) = column_q.find_by_id("property-panel") {
        match panel_state.state() {
            true => column.style.node.display = Display::default(),
            false => {
                column.style.node.display = Display::None;
                panel_state.reset();
            },
        }
    }
}

pub fn handle_update_property_panel_content(
    mut container_q: Query<(&Id, &Class, &mut Node), With<MakaraColumn>>,
    panel_state: Res<PropertyPanelShowState>,
) {
    if !panel_state.is_changed() {
        return;
    }

    let container_id = match panel_state.property_type {
        PropertyType::ReadCsv => "read-csv-property-container",
        PropertyType::ReplaceMissingValue => "replace-missing-value-property-container",
        _ => return
    };

    for (id, class, mut node) in container_q.iter_mut() {
        if class.value != "property-container" {
            continue;
        }

        if id.0 == container_id {
            node.display = Display::default();
        }
        else {
            node.display = Display::None;
        }
    }
}

pub fn property_panel() -> impl Bundle {
    column_!(
        id: "property-panel";

        [
            row_!(justify_content: JustifyContent::SpaceBetween; [
                text_!("Property", font_size: 14.0),
                button_!("x", class: "is-light"; on: on_close_button_clicked)
            ]),

            read_csv_property_container(),
            replace_missing_value_property_container()
        ]
    )
}


fn on_close_button_clicked(_: On<Clicked>, mut panel_state: ResMut<PropertyPanelShowState>) {
    panel_state.toggle();
}
