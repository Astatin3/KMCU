use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Kinematics {
    #[serde(rename = "core_xy")]
    CoreXY {
        #[serde(flatten)]
        general: GeneralKinematics,

        axis_x: [String; 2],
        axis_y: [String; 2],
        axis_z: [String; 2],

        axis_extruder: [String; 2],
    },
    // TODO: add support for more types
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
