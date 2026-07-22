use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    config::{axis::AxisConfig, sim_mcu::SimMCUConfig},
    runtime::axes::DummyAxis,
    traits::{axis::Axis, from_config::FromConfig, mcu::MCU},
};

pub struct SimMCURuntime {
    // axes: HashMap<String, Box<dyn Axis>>,
}

impl MCU for SimMCURuntime {
    fn new_axis(
        _this: Rc<RefCell<dyn MCU>>,
        axis_config: AxisConfig,
    ) -> anyhow::Result<Box<dyn Axis>>
    where
        Self: Sized,
    {
        // get what axis this is
        let axis = match axis_config {
            AxisConfig::Dummy => Box::new(DummyAxis::from_config(axis_config)?) as Box<dyn Axis>,

            _ => unimplemented!("TODO: Implement more motor types"),
        };

        Ok(axis)
    }
}

impl FromConfig for SimMCURuntime {
    type ConfigType = SimMCUConfig;

    fn from_config(config: SimMCUConfig) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {})
    }
}
