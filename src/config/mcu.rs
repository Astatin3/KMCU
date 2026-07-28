use crate::{
    config::{AxisConfig, connection_wrapper::ConnectionWrapper},
    utils::units,
};
use alloc::{collections::BTreeMap, string::String};
use serde::Deserialize;

use crate::config::connection::Connection;

#[derive(Debug, Deserialize)]
pub struct MCUConfig {
    pub connection: Connection,

    // Sometimes the host must power the MCU
    #[serde(default)]
    pub connection_wrapper: ConnectionWrapper,

    #[serde(default = "default_start_duration")]
    pub start_delay: units::LongTime,

    #[serde(flatten)]
    pub inner: MCUConfigType,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum MCUConfigType {
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
    pub axis: BTreeMap<String, AxisConfig>,
}
