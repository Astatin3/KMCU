use alloc::{boxed::Box, collections::BTreeMap, string::String};

use crate::traits::Axis;

/// A struct that contains mappings from device
/// names to their actual aces
#[derive(Default)]
pub struct DeviceMap {
    pub axes: BTreeMap<String, Box<dyn Axis>>,
}
