use bevy::prelude::*;
use uuid::Uuid;
use super::*;

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
