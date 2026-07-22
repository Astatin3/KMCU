use std::collections::HashMap;

use serde::Deserialize;

use crate::config::{axis::AxisConfig, connection::Connection, pin::Pin};

#[derive(Debug, Deserialize)]
pub struct KlipperMCU {
    pub connection: Connection,

    pub power_pin: Option<Pin>,
    // pub axes: HashMap<String, AxisConfig>,
}
