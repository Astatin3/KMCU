// use serde::;
mod axis;
mod connection;
mod kinematics;
mod mcu;
mod pin;

pub use axis::*;
pub use connection::*;
pub use kinematics::*;
pub use mcu::*;
pub use pin::*;

use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrinterConfig {
    pub kinematics: Kinematics,

    #[serde(default)]
    pub mcu: HashMap<String, MCUConfig>,
    #[serde(default)]
    pub axis: HashMap<String, AxisConfig>,
}

impl PrinterConfig {
    pub fn parse(config_string: &str) -> anyhow::Result<Self> {
        Ok(toml::from_str(config_string)?)
    }
}
