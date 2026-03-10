use bevy::input::{mouse::MouseButtonInput, ButtonState};
use bevy::window::{CursorIcon, SystemCursorIcon};
use bevy::prelude::*;
use makara::prelude::*;
use std::f32::consts::PI;
use uuid::Uuid;

use super::*;

pub struct CanvasPlugin;

impl Plugin for CanvasPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            handle_update_op_button_on_connect_flag_change_system,
            update_temp_connection_curve_system,
            draw_temp_gizmo_connection_curve_system,
            draw_connected_gizmo_curve_system,
            detect_mouse_press_to_cancel_connection_system,
            detect_mouse_over_connected_curves
        ));
    }
}

pub fn spawn_operator_entity(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    op: &Operator
) -> Entity {
    let rect = meshes.add(Rectangle::new(OPERATOR_SIZE.x, OPERATOR_SIZE.y));

    let op_entity = commands
        .spawn((
            Mesh2d(rect),
            MeshMaterial2d(materials.add(rgb(0.0, 0.0, 1.0))),
            Transform::from_xyz(100.0, 100.0, 0.0),
            OpBox::new(op.id, &op.name),
            op.clone()
        ))
        .observe(on_op_drag)
        .observe(on_op_right_clicked)
        .id();

    let mut input_button_entity: Option<Entity> = None;

    // if no input, no need to have input button
    match op.input {
        DataValue::None => {}
        _ => {
            let input_button = commands.spawn((
                Mesh2d(meshes.add(CircularSegment::new(10.0, 1.25))),
                MeshMaterial2d(materials.add(rgb(1.0, 1.0, 1.0))),
                Transform {
                    translation: Vec3::new(-(OPERATOR_SIZE.x / 2.0) + 2.0, 0.0, 0.0),
                    rotation: Quat::from_rotation_z(PI / 2.0),
                    ..default()
                },
                OpConnectButton::new_as_input(),
                OperatorEntity(op_entity),
                observe(on_input_button_clicked),
                observe(on_input_button_mouse_over),
                observe(on_input_button_mouse_out),
            ));
            input_button_entity = Some(input_button.id());
        }
    }

    let op_name_entity = commands.spawn((
        Text2d::new(&op.name),
        TextFont {
            font_size: 10.0,
            ..default()
        },
        Transform::from_xyz(0.0, 35.0, 0.0)
    ))
    .id();

    let output_button_entity = commands.spawn((
        Mesh2d(meshes.add(CircularSegment::new(10.0, 1.25))),
        MeshMaterial2d(materials.add(rgb(1.0, 1.0, 1.0))),
        Transform {
            translation: Vec3::new((OPERATOR_SIZE.x / 2.0) - 2.0, 0.0, 0.0),
            rotation: Quat::from_rotation_z(-PI / 2.0),
            ..default()
        },
        OpConnectButton::new_as_output(),
        OperatorEntity(op_entity),
        observe(on_output_button_clicked),
        observe(on_output_button_mouse_over),
        observe(on_output_button_mouse_out),
    ))
    .id();

    commands.entity(op_entity).add_children(&[op_name_entity, output_button_entity]);

    if let Some(input_button_entity) = input_button_entity {
        commands.entity(op_entity).add_child(input_button_entity);
    }

    op_entity
}

fn on_op_right_clicked(
    mut clicked: On<Pointer<Click>>,
    mut messages: MessageWriter<ToggleOpContext>,
    mut right_clicked_op_box: ResMut<RightClickedOperatorBox>,
    op_box: Query<&OpBox>,
    window: Query<&Window>
) {
    clicked.propagate(false);

    if let Some(cursor_pos) = get_ui_position_from_cursor(&window) {
        if clicked.button != PointerButton::Secondary {
            return;
        }

        if let Ok(op_box) = op_box.get(clicked.entity) {
            messages.write(ToggleOpContext::show(&op_box, &cursor_pos));
            right_clicked_op_box.0 = Some(op_box.id);
        }
    }
}

fn on_op_drag(
    _drag: On<Pointer<Drag>>,
    mut transforms: Query<&mut Transform, With<OpBox>>,
    window: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>
) {
    if let Ok(mut transform) = transforms.get_mut(_drag.entity) {
        if let Some(pos) = get_world_position_from_cursor(&window, &camera_q) {
            transform.translation.x = pos.x;
            transform.translation.y = pos.y;
        }
    }
}

fn on_output_button_clicked(
    mut clicked: On<Pointer<Click>>,
    mut state: ResMut<OpLineConnectionState>,
    connection_button: Query<&OpConnectButton>
) {
    if let Ok(connect_btn) = connection_button.get(clicked.entity) {
        state.reset();
        state.output_button_entity = Some(clicked.entity);
        state.output_button_type = connect_btn.button_type.clone();
    }

    clicked.propagate(false);
}

fn on_output_button_mouse_over(
    mut over: On<Pointer<Over>>,
    mut transforms: Query<&mut Transform, With<OpConnectButton>>
) {
    if let Ok(mut transform) = transforms.get_mut(over.entity) {
        transform.scale = Vec3::splat(1.2); // 20% larger
    }
    over.propagate(false);
}

fn on_output_button_mouse_out(
    mut over: On<Pointer<Out>>,
    mut transforms: Query<&mut Transform, With<OpConnectButton>>
) {
    if let Ok(mut transform) = transforms.get_mut(over.entity) {
        transform.scale = Vec3::ONE
    }
    over.propagate(false);
}

fn on_input_button_clicked(
    mut clicked: On<Pointer<Click>>,
    mut state: ResMut<OpLineConnectionState>,
    mut connected_curves: ResMut<ConnectedCurves>,
    mut operator_q: Query<&mut Operator>,
    mut commands: Commands,
    operator_button_q: Query<&OperatorEntity>,
    connection_button: Query<&OpConnectButton>
) {
    // construct final connnected curve and cancel the temp curve

    if let Ok(connect_btn) = connection_button.get(clicked.entity) {
        if state.output_button_entity.is_some() && state.input_button_is_hovering {
            state.input_button_entity = Some(clicked.entity);
            state.input_button_type = connect_btn.button_type.clone();

            let output_button_entity = state.output_button_entity.unwrap();

            connected_curves.0.push(Connection {
                id: Uuid::new_v4(),
                out_entity: output_button_entity,
                in_entity: clicked.entity,
            });

            let mut next_op_entity: Option<Entity> = None;

            if let Ok(op_entity) = operator_button_q.get(clicked.entity) {
                next_op_entity = Some(op_entity.0);
            }

            if let Ok(op_entity) = operator_button_q.get(output_button_entity) {
                if let Ok(mut op) = operator_q.get_mut(op_entity.0) {
                    op.is_first_operator = false; // mark it as false first

                    op.next_operator = next_op_entity;

                    // if an op doesn't have input data, mark it as head or first operator
                    if op.input == DataValue::None {
                        op.is_first_operator = true;
                    }
                }
            }

            state.reset();
        }
    }

    clicked.propagate(false);
}

fn on_input_button_mouse_over(
    mut over: On<Pointer<Over>>,
    mut transforms: Query<&mut Transform, With<OpConnectButton>>,
    mut state: ResMut<OpLineConnectionState>
) {
    if let Ok(mut transform) = transforms.get_mut(over.entity) {
        transform.scale = Vec3::splat(1.2); // 20% larger

        if state.output_button_entity.is_some() {
            state.input_button_is_hovering = true;
        }
    }
    over.propagate(false);
}

fn on_input_button_mouse_out(
    mut over: On<Pointer<Out>>,
    mut transforms: Query<&mut Transform, With<OpConnectButton>>,
    mut state: ResMut<OpLineConnectionState>
) {
    if let Ok(mut transform) = transforms.get_mut(over.entity) {
        transform.scale = Vec3::ONE;

        if state.output_button_entity.is_some() {
            state.input_button_is_hovering = false;
        }
    }
    over.propagate(false);
}

pub fn update_temp_connection_curve_system(
    state: Res<OpLineConnectionState>,
    query: Query<&GlobalTransform>,
    window: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut temp_curve: ResMut<TempCurveData>
) {
    let Some(entity) = state.output_button_entity else {
        temp_curve.cubic_curve = None;
        return;
    };

    if state.input_button_entity.is_some() {
        return;
    }

    let Some(mouse_pos) = get_world_position_from_cursor(&window, &camera_q) else {
        return;
    };

    if let Ok(transform) = query.get(entity) {
        let mut start_pos = transform.translation().truncate();
        start_pos.x += 10.0;

        let end_pos = mouse_pos;

        // Cardinal splines (Catmull-Rom) usually need 4 points to define a segment,
        // but because temp curve is just a straight line, we only need 2 points.
        let points = vec![start_pos, end_pos];
        let spline = CubicCardinalSpline::new_catmull_rom(points);

        temp_curve.cubic_curve = spline.to_curve().ok();
    }
}

// more info https://bevy.org/examples/math/cubic-splines/
pub fn draw_temp_gizmo_connection_curve_system(temp_curve: Res<TempCurveData>, mut gizmos: Gizmos) {
    let Some(ref curve) = temp_curve.cubic_curve else {
        return;
    };
    let resolution = 100 * curve.segments().len();
    gizmos.linestrip(
        curve.iter_positions(resolution).map(|pt| pt.extend(0.0)),
        Color::srgb(1.0, 1.0, 1.0),
    );
}

pub fn draw_connected_gizmo_curve_system(
    connected_curves: Res<ConnectedCurves>,
    transforms: Query<&GlobalTransform>,
    mut gizmos: Gizmos,
) {
    for conn in &connected_curves.0 {
        let Ok(out_tf) = transforms.get(conn.out_entity) else { continue };
        let Ok(in_tf) = transforms.get(conn.in_entity) else { continue };

        let spline = construct_connected_spline(
            out_tf.translation().truncate(),
            in_tf.translation().truncate()
        );

        if let Ok(curve) = spline.to_curve() {
            gizmos.linestrip(
                // break the curve into 50 pieces
                curve.iter_positions(50).map(|pt| pt.extend(0.0)), // Z=1.0 to stay on top
                Color::WHITE,
            );
        }
    }
}

pub fn detect_mouse_over_connected_curves(
    connected_curves: Res<ConnectedCurves>,
    transforms: Query<&GlobalTransform>,
    window: Query<&Window>,
    window_entity: Single<Entity, With<Window>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    asset_server: Res<AssetServer>,
    mut hovered_curve: ResMut<HoveredCurve>,
    mut commands: Commands
) {
    let Some(mouse_pos) = get_world_position_from_cursor(&window, &camera_q) else {
        return;
    };

    let hover_threshold = 6.0;
    let mut is_hovered = false;

    for connection in connected_curves.0.iter() {
        let Ok(out_tf) = transforms.get(connection.out_entity) else { continue };
        let Ok(in_tf) = transforms.get(connection.in_entity) else { continue };

        let spline = construct_connected_spline(
            out_tf.translation().truncate(),
            in_tf.translation().truncate()
        );

        if let Ok(curve) = spline.to_curve() {
            let positions: Vec<Vec2> = curve.iter_positions(50).collect();

            for window in positions.windows(2) {
                if distance_to_segment(mouse_pos, window[0], window[1]) < hover_threshold {
                    is_hovered = true;
                    break;
                }
            }

            if !is_hovered && hovered_curve.close_icon_entity.is_some() {
                let cursor_icon = CursorIcon::System(SystemCursorIcon::Default);
                commands.entity(*window_entity).insert(cursor_icon);

                if let Some(close_entity) = hovered_curve.close_icon_entity {
                    commands.entity(close_entity).despawn();
                    hovered_curve.reset();
                }
            }

            if is_hovered && hovered_curve.close_icon_entity.is_none() {
                let cursor_icon = CursorIcon::System(SystemCursorIcon::Pointer);
                commands.entity(*window_entity).insert(cursor_icon);

                let center_point = positions[25];

                if hovered_curve.close_icon_entity.is_none() {
                    let entity = commands.spawn((
                        Sprite {
                            image: asset_server.load("close_icon.png"),
                            custom_size: Some(Vec2::new(25.0, 25.0)),
                            image_mode: SpriteImageMode::Auto,
                            ..default()
                        },
                        Transform::from_translation(Vec3::new(center_point.x, center_point.y, 1.0)),
                        Pickable::default()
                    ))
                    .observe(handle_on_close_icon_clicked)
                    .id();

                    hovered_curve.close_icon_entity = Some(entity);
                    hovered_curve.id = Some(connection.id);
                }
            }
            // break;
        }
    }
}

fn handle_on_close_icon_clicked(
    mut clicked: On<Pointer<Click>>,
    mut hovered_curve: ResMut<HoveredCurve>,
    mut connected_curves: ResMut<ConnectedCurves>,
    mut commands: Commands,
    window_entity: Single<Entity, With<Window>>,
) {
    clicked.propagate(false);

    let Some(close_icon_entity) = hovered_curve.close_icon_entity else {
        return;
    };

    let Some(hovered_curve_id) = hovered_curve.id else {
        return;
    };

    if clicked.entity != close_icon_entity {
        return;
    }

    for (i, connection) in connected_curves.0.iter().enumerate() {
        if connection.id == hovered_curve_id {
            connected_curves.0.remove(i);
            hovered_curve.reset();

            let cursor_icon = CursorIcon::System(SystemCursorIcon::Default);
            commands.entity(close_icon_entity).despawn();
            commands.entity(*window_entity).insert(cursor_icon);
            break;
        }
    }
}

pub fn detect_mouse_press_to_cancel_connection_system(
    mut mouse_reader: MessageReader<MouseButtonInput>,
    mut state: ResMut<OpLineConnectionState>,
    mut temp_curve: ResMut<TempCurveData>
) {
    for mouse in mouse_reader.read() {
        if mouse.button != MouseButton::Left {
            continue;
        }

        match mouse.state {
            ButtonState::Pressed => {
                if !state.input_button_is_hovering {
                    state.reset();
                    temp_curve.cubic_curve = None;
                }
            }
            _ => {}
        }
    }
}

pub fn handle_update_op_button_on_connect_flag_change_system(
    mut op_btns: Query<
        (&OpConnectButton, &mut MeshMaterial2d<ColorMaterial>),
        Changed<OpConnectButton>
    >,
    mut materials: ResMut<Assets<ColorMaterial>>
) {
    for (op_btn, mut mesh_material) in op_btns.iter_mut() {
        if op_btn.connected {
            mesh_material.0 = materials.add(rgb(1.0, 1.0, 0.0));
        }
        else {
            mesh_material.0 = materials.add(rgb(1.0, 1.0, 1.0));
        }
    }
}
