use serde::Deserialize;

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

    pub axis_x: [String; 2],
    pub axis_y: [String; 2],
    pub axis_z: [String; 2],
    pub axis_extruder: [String; 2],
}

#[derive(Debug, Deserialize)]
pub struct GeneralKinematics {
    pub max_velocity: u32,
    pub max_accel: u32,
    pub max_z_velocity: u32,
    pub max_z_accel: u32,
    pub x_range: [u32; 2],
    pub y_range: [u32; 2],
    pub z_range: [u32; 2],
    pub square_corner_velocity: f32,
}
