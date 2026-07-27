use alloc::string::String;
use serde::Deserialize;

use crate::utils::units::LongTime;

/// Defines a way the MCU is powered on
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ConnectionWrapper {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "set_pin")]
    SetPin { pin: String },
    #[serde(rename = "pulse")]
    Pulse { pin: String, pulse: LongTime },
    #[serde(rename = "elegoo_a55a")]
    ElegooA55A { pin: String },
}

impl Default for ConnectionWrapper {
    fn default() -> Self {
        Self::None
    }
}
