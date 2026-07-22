use std::collections::HashMap;

use serde::Deserialize;

use crate::{config::axis::AxisConfig, traits::from_config::FromConfig};

#[derive(Debug, Deserialize)]
pub struct SimMCUConfig {
    // pub axes: HashMap<String, AxisConfig>,
}
