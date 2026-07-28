mod axis;
mod connection;
mod connection_wrapper;
mod kinematics;
mod mcu;

use alloc::{collections::btree_map::BTreeMap, string::String, vec::Vec};
pub use axis::*;
pub use connection::*;
pub use connection_wrapper::ConnectionWrapper;
pub use kinematics::*;
pub use mcu::*;

use crate::utils::error::{ConfigError, MajorStateError};

use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrinterConfig {
    pub kinematics: Kinematics,

    // Command to execute on startup.
    // Useful for configuring sockets and such
    pub exec_start: Option<String>,

    #[serde(default)]
    pub mcu: BTreeMap<String, MCUConfig>,
}

impl PrinterConfig {
    pub fn parse(config_string: &str) -> Result<Self, MajorStateError> {
        toml::from_str(config_string).map_err(|e| ConfigError::Toml(e).into())
    }
}
