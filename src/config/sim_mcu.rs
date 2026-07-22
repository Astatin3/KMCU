use std::collections::HashMap;

use serde::Deserialize;

use crate::config::axis::Axis;

#[derive(Debug, Deserialize)]
pub struct SimMCU {
    pub axes: HashMap<String, Axis>,
}
