mod axis;
mod connection;
mod kinematics;
mod mcu;
mod pin;

use alloc::{collections::btree_map::BTreeMap, string::String};
pub use axis::*;
pub use connection::*;
pub use kinematics::*;
pub use mcu::*;
// pub use pin::*;

use crate::Res;

use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrinterConfig {
    pub kinematics: Kinematics,

    #[serde(default)]
    pub mcu: BTreeMap<String, MCUConfig>,
    #[serde(default)]
    pub axis: BTreeMap<String, AxisConfig>,
}

impl PrinterConfig {
    pub fn parse(config_string: &str) -> Res<Self> {
        toml::from_str(config_string).map_err(|e| err!("{e}"))
    }
}
