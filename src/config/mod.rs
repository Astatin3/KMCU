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
    connection::Connection, kinematics::Kinematics, klipper_mcu::KlipperMCU, pin::Pin,
    sim_mcu::SimMCU,
};

#[derive(Debug, Deserialize)]
pub struct PrinterConfig {
    pub kinematics: Kinematics,

    #[serde(default)]
    pub klipper_mcu: HashMap<String, KlipperMCU>,
    #[serde(default)]
    pub sim_mcu: HashMap<String, SimMCU>,
}

impl PrinterConfig {
    pub fn parse(config_string: &str) -> anyhow::Result<Self> {
        Ok(toml::from_str(config_string)?)
    }
}
