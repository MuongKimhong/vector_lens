use bevy::prelude::*;
use makara::prelude::*;
use uuid::Uuid;

use super::*;

#[derive(Debug)]
pub enum PropertyType {
    None,
    ReadCsv,
    ReadExcel,
    AppendCsv,
    SelectAttributes,
    SaveCsvOrExcel,
    ReplaceMissingValue,
    NormalizerAndEncoder,
    TrainTestSplit,
    LinearRegression
}

#[derive(Resource, Debug)]
pub struct PropertyPanelShowState {
    pub state: bool,
    pub op_id: Option<Uuid>,
    pub op_name: Option<String>,
    pub op_entity: Option<Entity>,
    pub property_type: PropertyType,
}

impl PropertyPanelShowState {
    pub fn new() -> Self {
        Self {
            state: false,
            op_id: None,
            op_name: None,
            op_entity: None,
            property_type: PropertyType::None
        }
    }

    pub fn reset(&mut self) {
        self.state = false;
        self.op_id = None;
        self.op_name = None;
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
    mut container_q: Query<(&Id, &Class, &mut Node), (With<MakaraColumn>, Without<MakaraText>)>,
    mut text_q: TextQuery,
    panel_state: Res<PropertyPanelShowState>,
) {
    if !panel_state.is_changed() {
        return;
    }

    let container_id = match panel_state.property_type {
        PropertyType::ReadCsv => "read-csv-property-container",
        PropertyType::AppendCsv => "append-csv-property-container",
        PropertyType::ReadExcel => "read-excel-property-container",
        PropertyType::SelectAttributes => "select-attributes-property-container",
        PropertyType::SaveCsvOrExcel => "save-to-csv-or-excel-property-container",
        PropertyType::ReplaceMissingValue => "replace-missing-value-property-container",
        PropertyType::NormalizerAndEncoder => "normalizer-and-encoder-property-container",
        PropertyType::TrainTestSplit => "train-test-split-property-container",
        PropertyType::LinearRegression => "linear-regression-property-container",
        _ => return
    };

    for (id, class, mut node) in container_q.iter_mut() {
        if class.value != "property-container" {
            continue;
        }

        if id.0 == container_id {
            node.display = Display::default();

            if let Some(text) = text_q.find_by_id("property-op-name") {
                if let Some(name) = &panel_state.op_name {
                    text.text.value.0 = name.to_owned();
                }
            }
        }
        else {
            node.display = Display::None;
        }
    }
}

// use commands to spawn directly instead of return impl Bundle.
// because macro expansion is likely to cause god damn stack overflow. no idea why.
pub fn property_panel(commands: &mut Commands) -> Entity {
    commands
        .spawn(column_!(id: "property-panel"))
        .with_children(|parent| {
            parent.spawn(row_!(
                justify_content: JustifyContent::SpaceBetween; [
                    text_!("Property", font_size: 14.0),
                    button_!("x", class: "is-light"; on: on_close_button_clicked)
                ]
            ));

            parent.spawn(
                text_!("", id: "property-op-name", font_size: 14.0, justify_content: JustifyContent::Center)
            );

            parent
                .spawn(column_!(height: percent(100), padding_x: px(5), justify_content: JustifyContent::Start))
                .with_children(|column_parent| {
                    column_parent.spawn(read_csv_property_container());
                    column_parent.spawn(append_csv_property_container());
                    column_parent.spawn(select_attributes_property_container());
                    column_parent.spawn(save_to_csv_or_excel_property_container());
                    column_parent.spawn(replace_missing_value_property_container());
                    column_parent.spawn(normalizer_and_encoder_property_container());
                    column_parent.spawn(train_test_split_property_container());
                    column_parent.spawn(linear_regression_property_container());
                });
        })
        .id()
}

fn on_close_button_clicked(_: On<Clicked>, mut panel_state: ResMut<PropertyPanelShowState>) {
    panel_state.toggle();
}
