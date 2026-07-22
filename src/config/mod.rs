// use serde::;
pub mod axis;
pub mod connection;
pub mod kinematics;
pub mod klipper_mcu;
pub mod pin;
pub mod sim_mcu;

use std::collections::HashMap;

use serde::Deserialize;

use crate::config::{
    axis::AxisConfig, connection::Connection, kinematics::Kinematics, klipper_mcu::KlipperMCU,
    pin::Pin, sim_mcu::SimMCUConfig,
};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrinterConfig {
    pub kinematics: Kinematics,

    #[serde(default)]
    pub mcu: HashMap<String, MCUConfig>,
    #[serde(default)]
    pub axis: HashMap<String, AxisConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum MCUConfig {
    #[serde(rename = "sim")]
    Sim(SimMCUConfig),
    #[serde(rename = "klipper")]
    Klipper(KlipperMCU),
}

impl PrinterConfig {
    pub fn parse(config_string: &str) -> anyhow::Result<Self> {
        Ok(toml::from_str(config_string)?)
    }
}
