use std::time::Duration;

use serde::Deserialize;

use crate::config::{connection::Connection, de_duration};

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

fn default_start_duration() -> Duration {
    Duration::ZERO
}

#[derive(Debug, Deserialize)]
pub struct KlipperMCU {
    pub connection: Connection,

    // Command to execute on startup.
    // Useful for configuring sockets and such
    pub exec_start: Option<String>,

    pub power_pin: Option<String>,

    #[serde(default = "default_start_duration", deserialize_with = "de_duration")]
    pub start_delay: Duration,
}
