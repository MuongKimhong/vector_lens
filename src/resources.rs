use crossbeam_channel::{unbounded, Receiver, Sender};
use serde::{Deserialize, Serialize};
use bevy::prelude::*;
use makara::prelude::*;
use chrono::Local;
use uuid::Uuid;

use std::{collections::HashMap, path::PathBuf};
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
    pub is_running: bool,
    pub executing_operator: Option<Entity>
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

#[derive(Resource, Debug)]
pub struct OperatorList(pub Vec<Operator>);

impl OperatorList {
    fn create_default_operators() -> Vec<Operator> {
        vec![
            read_csv_operator(),
            replace_missing_value_operator()
        ]
    }

    pub fn new() -> Self {
        Self(OperatorList::create_default_operators())
    }
}

#[derive(Resource, Debug, Default)]
pub struct OperatorInUseList(pub Vec<Operator>);

#[derive(Resource, Debug, Default)]
pub struct OpLineConnectionState {
    /// Connection always started with output button
    pub output_button_entity: Option<Entity>,
    pub output_button_type: OpConnectButtonType,

    /// Connection always ended with output button
    pub input_button_entity: Option<Entity>,
    pub input_button_type: OpConnectButtonType,
    pub input_button_is_hovering: bool
}

impl OpLineConnectionState {
    pub fn reset(&mut self) {
        *self = OpLineConnectionState::default();
    }
}

/// The curve presently being displayed. This is optional because there may not be enough control
/// points to actually generate a curve.
#[derive(Clone, Default, Resource)]
pub struct TempCurveData {
    pub id: Uuid,
    pub cubic_curve: Option<CubicCurve<Vec2>>,
}

pub struct Connection {
    pub id: Uuid,
    pub out_entity: Entity,
    pub in_entity: Entity,
}

/// The final curves that connected from one op box to another
#[derive(Resource, Default)]
pub struct ConnectedCurves(pub Vec<Connection>);

#[derive(Resource, Default)]
pub struct HoveredCurve {
    pub id: Option<Uuid>,
    pub close_icon_entity: Option<Entity>
}

impl HoveredCurve {
    pub fn reset(&mut self) {
        self.id = None;
        self.close_icon_entity = None;
    }
}

#[derive(Debug)]
pub enum LogType {
    Normal(String),
    Success(String),
    Error(String)
}

impl Default for LogType {
    fn default() -> Self {
        LogType::Normal("".to_string())
    }
}

/// Resource to hold log message.
/// Use hashmap as a session, when running a process, create new session,
/// which increase key count.
#[derive(Resource, Debug)]
pub struct ConsoleLog {
    pub messages: HashMap<usize, Vec<LogType>>,
    pub last_key_count: usize
}

impl Default for ConsoleLog {
    fn default() -> Self {
        let mut messages: HashMap<usize, Vec<LogType>> = HashMap::new();
        let last_key_count = 1;
        let log = format!(
            "[LOG][{}] Application started",
            Local::now().format("%H:%M:%S")
        );

        messages.insert(last_key_count, vec![LogType::Normal(log)]);
        Self {
            messages,
            last_key_count
        }
    }
}

impl ConsoleLog {
    pub fn new_session(&mut self) {
        self.last_key_count += 1;
        self.messages.insert(self.last_key_count, Vec::new());
    }

    pub fn new_message(&mut self, log: LogType) {
        if let Some(messages) = self.messages.get_mut(&self.last_key_count) {
            messages.push(log);
        }
    }
}

#[derive(Debug)]
pub enum TaskChannelEvent {
    LogMessage(LogType),
    None
}

/// Receiver that listening to event from other threads created by AsyncComputeTaskPool
#[derive(Resource, Debug)]
pub struct TaskChannelReceiver(pub Receiver<TaskChannelEvent>);

#[derive(Resource, Debug)]
pub struct TaskChannelSender(pub Sender<TaskChannelEvent>);

/// Resource used to keep track of process file.
/// - Is user editing an existing process?
/// - Is user using application without any process file?
#[derive(Resource, Debug, Default)]
pub struct ProcessFileState {
    pub editing_existing_process: bool,
    pub currernt_process_path: Option<PathBuf>,
    pub file_needs_to_be_saved: bool
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OperatorFormat {
    pub transform_x: f32,
    pub transform_y: f32,
    pub op_object: Operator
}

impl OperatorFormat {
    pub fn new(translation: Vec2, op: &Operator) -> Self {
        Self {
            transform_x: translation.x,
            transform_y: translation.y,
            op_object: op.clone()
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct ProcessFileFormat {
    pub operators: Vec<OperatorFormat>,
    pub file_name: String
}

/// A resource used to store the path of selected destination
/// when user want to save new process (Save process as).
#[derive(Resource, Debug, Default)]
pub struct SaveProcessAsBackgroundThreadReceiver {
    pub receiver: Option<Receiver<Option<PathBuf>>>
}
