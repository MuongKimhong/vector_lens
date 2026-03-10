pub mod messages;
pub mod category;
pub mod op;

pub use messages::*;
pub use category::*;
pub use op::*;

use bevy::prelude::*;
use makara::prelude::*;
use uuid::Uuid;

use std::collections::HashMap;
use crate::resources::*;

pub struct OperatorPlugin;

impl Plugin for OperatorPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ToggleOpContext>();
    }
}

#[derive(Debug, Default, Clone)]
pub enum DataValue {
    #[default]
    None,
    Csv(String),
    FilePath(String),
    Table,
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
pub struct Operator {
    pub id: Uuid,
    pub name: String,
    pub input: DataValue,
    pub output: DataValue,
    pub category: OperatorCategory,
    pub kind: OperatorKind,
    pub entity: Option<Entity>,
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
            entity: None,
        }
    }

    /// Create a new operator from an existing operator with new id.
    pub fn new_from(op: &Operator) -> Self {
        let mut new_op = op.clone();
        new_op.id = Uuid::new_v4();
        new_op
    }
}
