use bevy::prelude::*;
use chrono::Local;
use std::path::PathBuf;
use std::fs::File;
use std::io::{BufWriter, Write};

use crate::resources::LogType;

pub fn get_world_position_from_cursor(
    windows: &Query<&Window>,
    camera_q: &Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let (camera, camera_transform) = camera_q.single().ok()?;
    let window = windows.single().ok()?;
    let cursor_position = window.cursor_position()?;

    camera
        .viewport_to_world_2d(camera_transform, cursor_position)
        .ok()
}

pub fn get_ui_position_from_cursor(windows: &Query<&Window>) -> Option<Vec2> {
    let window = windows.single().ok()?;
    window.cursor_position()
}

pub fn get_ui_position_from_world_position(
    camera_q: &Query<(&Camera, &GlobalTransform)>,
    world_position: Vec3
) -> Option<Vec2> {
    let (camera, camera_transform) = camera_q.single().ok()?;
    camera
        .world_to_viewport(camera_transform, world_position)
        .ok()
}

// check the distance between the given point (mouse) to the line from a to b
pub fn distance_to_segment(point: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let ap = point - a;
    let length_sq = ab.length_squared();

    if length_sq == 0.0 {
        return ap.length();
    }

    let t = (ap.dot(ab) / length_sq).clamp(0.0, 1.0);
    let closest_point = a + t * ab;
    (point - closest_point).length()
}

// create points and construct a CubicCardinalSpline from 2 points (2 translations);
pub fn construct_connected_spline(start: Vec2, end: Vec2) -> CubicCardinalSpline<Vec2> {
    let dx = (end.x - start.x).abs();

    let adaptive_offset = if dx < 30.0 {
        0.0
    } else {
        ((dx - 30.0) * 0.5).min(150.0)
    };

    let p1 = start + Vec2::new(10.0, 0.0);
    let p2 = start + Vec2::new(adaptive_offset, 0.0);
    let p3 = end + Vec2::new(-adaptive_offset, 0.0);
    let p4 = end + Vec2::new(-10.0, 0.0);

    let points = vec![p1, p2, p3, p4];
    CubicCardinalSpline::new_catmull_rom(points)
}

pub fn create_log_with_timestamp(msg: &str) -> String {
    format!("[{}] {}", Local::now().format("%H:%M:%S"), msg)
}

pub fn log_error(msg: &str) -> LogType {
    LogType::Error(create_log_with_timestamp(msg))
}

pub fn log_normal(msg: &str) -> LogType {
    LogType::Normal(create_log_with_timestamp(msg))
}

pub fn log_success(msg: &str) -> LogType {
    LogType::Success(create_log_with_timestamp(msg))
}

pub fn write_file(path: PathBuf, data: &[u8]) -> std::io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    writer.write_all(data)?;
    Ok(())
}

pub fn is_control_key_held(keys: &Res<ButtonInput<KeyCode>>) -> bool {
    #[cfg(target_os = "macos")]
    {
        keys.pressed(KeyCode::SuperLeft) || keys.pressed(KeyCode::SuperRight)
    }
    #[cfg(not(target_os = "macos"))]
    {
        keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight)
    }
}
