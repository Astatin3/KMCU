use crate::units;
use alloc::string::String;
use serde::Deserialize;

use crate::config::connection::Connection;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum MCUConfig {
    #[serde(rename = "sim")]
    Sim(SimMCUConfig),
    #[serde(rename = "klipper")]
    Klipper(KlipperMCU),
}

#[derive(Debug, Deserialize)]
pub struct SimMCUConfig {}

fn default_start_duration() -> units::LongTime {
    units::LongTime::new::<units::long_millisecond>(0)
}

#[derive(Debug, Deserialize)]
pub struct KlipperMCU {
    pub connection: Connection,

    // Command to execute on startup.
    // Useful for configuring sockets and such
    pub exec_start: Option<String>,

    // Sometimes the host must power the MCU via GPIO
    pub power_pin: Option<String>,

    #[serde(default = "default_start_duration")]
    pub start_delay: units::LongTime,
}
