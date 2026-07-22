use std::{cell::RefCell, collections::HashMap, rc::Rc};

use anyhow::anyhow;

use crate::{
    config::{axis::AxisConfig, kinematics::CoreXYKinematics},
    runtime::axes::DummyAxis,
    traits::{axis::Axis, from_config::FromConfig, mcu::MCU},
};

pub struct CoreXYRuntime {
    config: CoreXYKinematics,
    mcus: HashMap<String, Rc<RefCell<dyn MCU>>>,

    axis_x: Box<dyn Axis>,
    axis_y: Box<dyn Axis>,
    axis_z: Box<dyn Axis>,
    axis_extruder: Box<dyn Axis>,
}

impl FromConfig for CoreXYRuntime {
    type ConfigType = (
        CoreXYKinematics,
        HashMap<String, AxisConfig>,
        HashMap<String, Rc<RefCell<dyn MCU>>>,
    );

    fn from_config((config, mut axes, mut mcus): Self::ConfigType) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let mut create_axis = |names: &(String, String)| -> anyhow::Result<Box<dyn Axis>> {
            let (mcu_name, axis_name) = names;

            let mcu = mcus
                .get(mcu_name)
                .ok_or(anyhow!("Could not find MCU by name of {mcu_name}"))?
                .clone();

            let axis_config = axes
                .remove(axis_name)
                .ok_or(anyhow!("Could not find axis by name of {axis_name}"))?;

            let axis = match axis_config {
                AxisConfig::Dummy => {
                    Box::new(DummyAxis::from_config(axis_config)?) as Box<dyn Axis>
                }

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
