use std::collections::HashMap;

use crate::{
    config::kinematics::CoreXYKinematics,
    traits::{from_config::FromConfig, mcu::MCU},
};

pub struct CoreXYRuntime {}

// impl FromConfig for CoreXYRuntime {
//     type ConfigType = (CoreXYKinematics, HashMap<String, Box<dyn MCU>>);

//     fn from_config((config, mcus): &Self::ConfigType) -> anyhow::Result<Self>
//     where
//         Self: Sized,
//     {
//         todo!()
//     }
// }
