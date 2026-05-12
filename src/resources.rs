use crossbeam_channel::{Receiver, Sender};
use bevy::prelude::*;
use makara::prelude::*;
use chrono::Local;
use uuid::Uuid;

use std::collections::HashMap;
use std::path::PathBuf;
use super::*;

#[derive(Default, Debug, PartialEq)]
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

    pub fn on_viewing_tab_change_system(
        state: Res<UiState>,
        mut btn_q: ButtonQuery,
        mut commands: Commands,
        design_page_root_entity: Res<DesignPageRootEntity>,
        result_page_root_entity: Res<ResultPageRootEntity>
    ) {
        if !state.is_changed() { return; }

        let (design_class, result_class) = match state.viewing_tab {
            ViewingTab::Design => ("is-primary-dark", "is-light"),
            ViewingTab::Result => ("is-light", "is-primary-dark")
        };

        let (design_vis, result_vis, design_trans_z, result_trans_z) = match state.viewing_tab {
            ViewingTab::Design => (Visibility::Visible, Visibility::Hidden, 1.0 as f32, 0.0 as f32),
            ViewingTab::Result => (Visibility::Hidden, Visibility::Visible, 0.0 as f32, 1.0 as f32)
        };

        if let Some(design_page_entity) = design_page_root_entity.get() {
            commands
                .entity(*design_page_entity)
                .insert((design_vis, Transform::from_translation(Vec3::new(0.0, 0.0, design_trans_z))));
        }

        if let Some(result_page_entity) = result_page_root_entity.get() {
            commands
                .entity(*result_page_entity)
                .insert((result_vis, Transform::from_translation(Vec3::new(0.0, 0.0, result_trans_z))));
        }

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
            append_csv_operator(),
            select_attributes_operator(),
            save_csv_or_excel_operator(),
            replace_missing_value_operator(),
            normalizer_and_encoder_operator(),
            train_test_split_operator(),
            linear_regression_operator()
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
            "[{}] Application started",
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
    SetTestData(DataValue),
    UpdatePreReadContentAfterSelectAttributes(DataValue),
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

impl ProcessFileState {
    pub fn reset(&mut self) {
        self.editing_existing_process = false;
        self.currernt_process_path = None;
        self.file_needs_to_be_saved = false;
    }

    pub fn can_save(&self) -> bool {
        self.editing_existing_process && self.currernt_process_path.is_some()
    }
}

/// A resource used to store the path of selected destination
/// when user want to save new process (Save process as).
#[derive(Resource, Debug, Default)]
pub struct SaveProcessAsBackgroundThreadReceiver {
    pub destination_receiver: Option<Receiver<Option<PathBuf>>>,
    pub save_result_receiver: Option<Receiver<std::io::Result<()>>>,
}

#[derive(Resource, Debug, Default)]
pub struct OpenProcessBackgroundThreadReceiver {
    pub choose_file_receiver: Option<Receiver<Option<PathBuf>>>,
    pub open_result_receiver: Option<Receiver<Result<ProcessFileFormat, serde_json::Error>>>
}

/// A resource used to store the path of selected destination
/// when user want to save to csv or excel using "Save to Csv or Excel" operator
#[derive(Resource, Debug, Default)]
pub struct SaveCsvOrExcelBackgroundThreadReceiver {
    pub destination_receiver: Option<Receiver<Option<PathBuf>>>,
}

/// A resource used as background task thread receiver, to get dataframe
/// content of "Read Csv" or "Read Excel". This resource is for this concept:
///
/// "Create a resource to hold dataframe of read csv and read excel.
/// When user set file path property in these 2 operators, spawn
/// a background task to pre read the content and put into the resource."
#[derive(Resource, Debug, Default, Getter)]
pub struct PreReadCsvOrExcelContentThreadReceiver(pub Option<Receiver<DataValue>>);

#[derive(Resource, Debug, Default, Getter)]
pub struct PreReadCsvOrExcelContent(pub DataValue);

#[derive(Resource, Debug, Default, Getter)]
pub struct DesignPageRootEntity(pub Option<Entity>);

#[derive(Resource, Debug, Default, Getter)]
pub struct ResultPageRootEntity(pub Option<Entity>);

/// This resource is used to store dataframe for test set
/// from train_test_split_operator
#[derive(Resource, Debug, Default, Getter)]
pub struct TestSet(pub Option<DataValue>);
