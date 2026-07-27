use alloc::string::String;
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

    pub axis_x: (String, String),
    pub axis_y: (String, String),
    pub axis_z: (String, String),
    pub axis_extruder: (String, String),
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
