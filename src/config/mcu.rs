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

#[derive(Debug, Deserialize)]
pub struct KlipperMCU {
    pub connection: Connection,

    // Command to execute on startup.
    // Useful for configuring sockets and such
    pub exec_start: Option<String>,

    pub power_pin: Option<String>,
}
