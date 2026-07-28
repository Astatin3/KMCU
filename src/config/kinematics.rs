use alloc::string::{String, ToString};
use serde::Deserialize;

use crate::utils::units::{self, Acceleration, Length, Velocity, velocity::millimeter_per_second};

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Kinematics {
    #[serde(rename = "core_xy")]
    CoreXY(CoreXYKinematics),
    // TODO: add support for more types
}
#[derive(Debug, Deserialize)]
pub struct CoreXYKinematics {
    #[serde(flatten)]
    pub general: GeneralKinematics,

    // Names for the axes of the printer
    #[serde(default = "default_axis_x")]
    pub axis_x: String,
    #[serde(default = "default_axis_y")]
    pub axis_y: String,
    #[serde(default = "default_axis_z")]
    pub axis_z: String,
    #[serde(default = "default_axis_e")]
    pub axis_extruder: String,
}

fn default_axis_x() -> String {
    "axis_x".into()
}
fn default_axis_y() -> String {
    "axis_y".into()
}
fn default_axis_z() -> String {
    "axis_z".into()
}
fn default_axis_e() -> String {
    "axis_e".into()
}

#[derive(Debug, Deserialize)]
pub struct GeneralKinematics {
    pub max_velocity: Velocity,
    pub max_accel: Acceleration,
    pub max_z_velocity: Velocity,
    pub max_z_accel: Acceleration,

    pub x_range: [Length; 2],
    pub y_range: [Length; 2],
    pub z_range: [Length; 2],
    pub square_corner_velocity: Velocity,
}
