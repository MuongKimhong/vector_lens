pub mod messages;
pub mod category;
pub mod op;

pub use messages::*;
pub use category::*;
pub use op::*;

use bevy::tasks::{AsyncComputeTaskPool, Task};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use bevy::prelude::*;
use makara::prelude::*;
use crossbeam_channel::{unbounded, Sender};
use uuid::Uuid;
use polars::prelude::*;

use std::collections::HashMap;
use crate::resources::*;

pub struct OperatorPlugin;

impl Plugin for OperatorPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ToggleOpContext>();
        app.add_systems(
            Update,
            (
                handle_op_background_execution_system,
                handle_insert_task_channel_resource_system,
                listen_to_task_channel_receiver_system
            )
        );
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub enum DataValue {
    #[default]
    None,
    Csv(String),
    FilePath(String),
    Table(DataFrame),
    Model
}

#[derive(Debug, Clone)]
pub enum OperatorCategory {
    IO,
    Cleaning,
    MachineLearning,
    DeepLearning
}

#[derive(Debug, Clone, Default)]
pub enum PropertyValue {
    #[default]
    None,
    String(String),
    Int(i32),
    Float(f32),
    Bool(bool),
    List(Vec<PropertyValue>),
}

#[derive(Debug, Clone)]
pub enum OperatorKind {
    ReadCSV,
    ReplaceMissingValue
}

#[derive(Component, Debug, Clone)]
pub struct OperatorName(pub String);

// Component hold running task of an operator.
#[derive(Component)]
pub struct ProcessingTask(pub Task<DataValue>);

// Component for holding entity of an operator.
// This component is used in Operator Input and Output buttons.
#[derive(Component, Debug, Clone)]
pub struct OperatorEntity(pub Entity);

// Component for holding text2d entity of an operator.
// This component is used in Operator entity.
#[derive(Component, Debug, Clone)]
pub struct OperatorNameEntity(pub Entity);

// An operator acts like a linked-list. It contains the next operator entity.
// It's easy to know which operator will get executed next.
#[derive(Component, Debug, Clone)]
pub struct Operator {
    pub id: Uuid,
    pub name: String,
    pub input: DataValue,
    pub output: DataValue,
    pub category: OperatorCategory,
    pub kind: OperatorKind,
    pub entity: Option<Entity>,
    pub next_operator: Option<Entity>,
    pub is_first_operator: bool,
    pub properties: HashMap<String, PropertyValue>
}

impl Operator {
    pub fn new(
        name: &str,
        kind: OperatorKind,
        input: DataValue,
        output: DataValue,
        category: OperatorCategory,
        properties: HashMap<String, PropertyValue>
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            kind,
            input,
            output,
            category,
            properties,
            next_operator: None,
            is_first_operator: false,
            entity: None,
        }
    }

    /// Create a new operator from an existing operator with new id.
    pub fn new_from(op: &Operator) -> Self {
        let mut new_op = op.clone();
        new_op.id = Uuid::new_v4();
        new_op
    }

    // Spawn operator execution task into bevy background computation.
    // https://bevy-cheatbook.github.io/fundamentals/async-compute.html
    pub fn spawn_task(&self, task_sender: &Sender<TaskChannelEvent>) -> Task<DataValue> {
        let thread_pool = AsyncComputeTaskPool::get();

        // We MUST clone the data the background thread needs.
        // You cannot pass `&mut self` into a background thread!
        let kind = self.kind.clone();
        let input_data = self.input.clone();
        let properties = self.properties.clone();
        let sender = task_sender.clone();

        thread_pool.spawn(async move {
            match kind {
                OperatorKind::ReadCSV => handle_read_csv_operator_execution(&sender, &properties),
                OperatorKind::ReplaceMissingValue => {
                    // Do math on `input_data` based on `properties`
                    println!("start executing replace missing value");
                    std::thread::sleep(std::time::Duration::from_secs(5));
                    println!("finished executing replace missing value");
                    DataValue::Table(DataFrame::empty())
                }
                // ...
            }
        })
    }

    pub fn execute(&mut self) {
        println!("executing {:?}", self.name);
    }
}
