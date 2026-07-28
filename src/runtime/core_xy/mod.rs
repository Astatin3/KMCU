use core::cell::RefCell;

use alloc::{boxed::Box, collections::btree_map::BTreeMap, rc::Rc, string::String, vec::Vec};

use crate::{
    config::{AxisConfig, CoreXYKinematics},
    runtime::{
        device_map::{self, DeviceMap},
        dummy::DummyAxis,
    },
    traits::{Axis, MCU},
    utils::{
        error::{IOError, MCUError, RuntimeError, RuntimeInitError},
        units::{self, Velocity},
    },
};

pub struct CoreXYRuntime {
    config: CoreXYKinematics,
    mcus: Vec<(String, Rc<RefCell<dyn MCU>>)>,

    axis_x: Box<dyn Axis>,
    axis_y: Box<dyn Axis>,
    axis_z: Box<dyn Axis>,
    axis_extruder: Box<dyn Axis>,
}

impl CoreXYRuntime {
    pub fn alive(&mut self) -> Result<(), RuntimeError> {
        for (name, mcu) in &mut self.mcus {
            mcu.borrow_mut()
                .alive()
                .map_err(|e| RuntimeError::MCU(MCUError::AliveCheckFailed(e), (name).into()))?;
        }

        Ok(())
    }

    pub fn from_config(
        config: CoreXYKinematics,
        mut device_map: DeviceMap,
        mcus: Vec<(String, Rc<RefCell<dyn MCU>>)>,
    ) -> Result<Self, RuntimeInitError>
    where
        Self: Sized,
    {
        debug!("Kinematics config: {config:?}");

        let mut get_axis = |axis_name: &str| -> Result<Box<dyn Axis>, RuntimeInitError> {
            let axis = device_map
                .axes
                .remove(axis_name)
                .ok_or(RuntimeInitError::CoreXY(MCUError::Axis(
                    IOError::NotConnected,
                    axis_name.into(),
                )))?;

            Ok(axis)
        };

        Ok(Self {
            axis_x: get_axis(&config.axis_x)?,
            axis_y: get_axis(&config.axis_y)?,
            axis_z: get_axis(&config.axis_z)?,
            axis_extruder: get_axis(&config.axis_extruder)?,

            mcus,
            config,
        })
    }

    pub fn test_x(&mut self) {
        self.axis_x
            .simple_move(Velocity::new::<units::millimeter_per_second>(1));
    }
}
