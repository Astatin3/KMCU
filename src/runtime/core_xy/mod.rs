use core::cell::RefCell;
use std::collections::HashMap;

use alloc::{boxed::Box, rc::Rc, string::String};

use crate::{
    config::{AxisConfig, CoreXYKinematics},
    error::Res,
    runtime::axes::DummyAxis,
    traits::{Axis, FromConfig, MCU},
};

pub struct CoreXYRuntime {
    config: CoreXYKinematics,
    mcus: HashMap<String, Rc<RefCell<dyn MCU>>>,

    axis_x: Box<dyn Axis>,
    axis_y: Box<dyn Axis>,
    axis_z: Box<dyn Axis>,
    axis_extruder: Box<dyn Axis>,
}

impl CoreXYRuntime {
    pub fn alive(&self) -> Res<()> {
        for (name, mcu) in &self.mcus {
            mcu.borrow_mut()
                .alive()
                .map_err(|e| err!("Failed to run alive check for MCU '{name}': {e}"))?;
        }

        Ok(())
    }
}

impl FromConfig for CoreXYRuntime {
    type ConfigType = (
        CoreXYKinematics,
        HashMap<String, AxisConfig>,
        HashMap<String, Rc<RefCell<dyn MCU>>>,
    );

    fn from_config((config, mut axes, mcus): Self::ConfigType) -> Res<Self>
    where
        Self: Sized,
    {
        let mut create_axis = |names: &(String, String)| -> Res<Box<dyn Axis>> {
            let (mcu_name, axis_name) = names;

            let mcu = mcus
                .get(mcu_name)
                .ok_or(err!("Could not find MCU by name of {mcu_name}"))?
                .clone();

            let axis_config = axes
                .remove(axis_name)
                .ok_or(err!("Could not find axis by name of {axis_name}"))?;

            let axis = match axis_config {
                AxisConfig::Dummy(dummy_axis_config) => DummyAxis::new(dummy_axis_config),

                _ => todo!(),
            };

            Ok(axis)
        };

        Ok(Self {
            axis_x: create_axis(&config.axis_x)?,
            axis_y: create_axis(&config.axis_y)?,
            axis_z: create_axis(&config.axis_z)?,
            axis_extruder: create_axis(&config.axis_extruder)?,

            mcus,
            config,
        })
    }
}
