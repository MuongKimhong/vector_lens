pub mod ui;
pub mod io;
pub mod canvas;
pub mod operators;
pub mod utils;
pub mod resources;
pub mod messages;

pub use io::*;
pub use ui::*;
pub use canvas::*;
pub use operators::*;
pub use utils::*;
pub use resources::*;
pub use messages::*;

use bevy::prelude::*;

#[derive(Component)]
pub struct DesignPageRoot;

#[derive(Component)]
pub struct ResultPageRoot;

fn main() {
    App::new()
        .add_plugins((OperatorPlugin, CanvasPlugin, UiPlugin, IoPlugin, MeshPickingPlugin))
        .insert_resource(TempCurveData::default())
        .insert_resource(ConnectedCurves::default())
        .insert_resource(OperatorList::new())
        .insert_resource(OperatorInUseList::default())
        .insert_resource(OpLineConnectionState::default())
        .insert_resource(HoveredCurve::default())
        .insert_resource(PreReadCsvOrExcelContent::default())
        .insert_resource(DesignPageRootEntity::default())
        .insert_resource(ResultPageRootEntity::default())
        .add_systems(Startup, (spawn_design_page_root, spawn_result_page_root))
        .run();
}

fn spawn_design_page_root(
    mut commands: Commands,
    mut design_page_root_entity: ResMut<DesignPageRootEntity>
) {
    let entity = commands.spawn((
        DesignPageRoot,
        Visibility::Visible,
        Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
    ))
    .id();

    design_page_root_entity.0 = Some(entity);
}

fn spawn_result_page_root(
    mut commands: Commands,
    mut result_page_root_entity: ResMut<ResultPageRootEntity>
) {
    let entity = commands.spawn((
        ResultPageRoot,
        Visibility::Hidden,
        Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
    ))
    .id();

    result_page_root_entity.0 = Some(entity);
}
