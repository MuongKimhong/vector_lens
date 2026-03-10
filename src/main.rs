pub mod ui;
pub mod canvas;
pub mod operators;
pub mod utils;
pub mod resources;

pub use ui::*;
pub use canvas::*;
pub use operators::*;
pub use utils::*;
pub use resources::*;

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins((OperatorPlugin, CanvasPlugin, UiPlugin, MeshPickingPlugin))
        .insert_resource(TempCurveData::default())
        .insert_resource(ConnectedCurves::default())
        .insert_resource(OperatorList::new())
        .insert_resource(OperatorInUseList::default())
        .insert_resource(OpLineConnectionState::default())
        .insert_resource(HoveredCurve::default())
        .run();
}
