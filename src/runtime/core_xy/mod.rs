use core::cell::RefCell;

use alloc::{boxed::Box, collections::btree_map::BTreeMap, rc::Rc, string::String};

use crate::{
    config::{AxisConfig, CoreXYKinematics},
    runtime::dummy::DummyAxis,
    traits::{Axis, MCU},
    utils::error::{IOError, MCUError, RuntimeError},
};

pub struct CoreXYRuntime {
    config: CoreXYKinematics,
    mcus: BTreeMap<String, Rc<RefCell<dyn MCU>>>,

    axis_x: Box<dyn Axis>,
    axis_y: Box<dyn Axis>,
    axis_z: Box<dyn Axis>,
    axis_extruder: Box<dyn Axis>,
}

impl CoreXYRuntime {
    pub fn alive(&self) -> Result<(), RuntimeError> {
        for (name, mcu) in &self.mcus {
            mcu.borrow_mut()
                .alive()
                .map_err(|e| RuntimeError::MCU(e, name.into()))?;
        }

        Ok(())
    }

    pub fn from_config(
        (config, mut axes, mcus): (
            CoreXYKinematics,
            BTreeMap<String, AxisConfig>,
            BTreeMap<String, Rc<RefCell<dyn MCU>>>,
        ),
    ) -> Result<Self, RuntimeError>
    where
        Self: Sized,
    {
        debug!("Kinematics config: {config:?}");

        let mut create_axis = |names: &(String, String)| -> Result<Box<dyn Axis>, RuntimeError> {
            let (mcu_name, axis_name) = names;

            let mcu = mcus
                .get(mcu_name)
                .ok_or(RuntimeError::MCU(
                    MCUError::KlipperConnection(IOError::NotConnected),
                    (mcu_name).into(),
                ))?
                .clone();

            let axis_config = axes.remove(axis_name).ok_or(RuntimeError::MCU(
                MCUError::KlipperConnection(IOError::NotConnected),
                (axis_name).into(),
            ))?;

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
