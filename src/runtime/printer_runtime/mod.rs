use core::cell::RefCell;

use alloc::{boxed::Box, collections::btree_map::BTreeMap, rc::Rc, vec::Vec};

use crate::{
    config::{Kinematics, MCUConfig, MCUConfigType, PrinterConfig},
    runtime::{
        connection::init_connection_with_wrapper, core_xy::CoreXYRuntime, device_map::DeviceMap,
        dummy::SimMCURuntime, klipper_mcu::KlipperMCURuntime,
    },
    traits::MCU,
    utils::error::{MCUError, RuntimeError, RuntimeInitError},
};

pub struct PrinterRuntime {
    pub kinematics: CoreXYRuntime,
}

impl PrinterRuntime {
    pub fn alive(&mut self) -> Result<(), RuntimeError> {
        self.kinematics.alive()
    }

    pub fn from_config(config: PrinterConfig) -> Result<Self, RuntimeInitError>
    where
        Self: Sized,
    {
        #[cfg(feature = "std")]
        if let Some(command) = config.exec_start {
            match std::process::Command::new("/bin/sh")
                .arg("-c")
                .arg(&command)
                .output()
            {
                Ok(o) => {
                    if o.status.success() {
                        info!("Ran command '{command}'")
                    } else {
                        warn!("Command '{command}' resulted in error {}", o.status);
                    }
                }
                Err(_) => warn!("Command '{command}' failed to run!"),
            }
        }

        let mut device_map = DeviceMap::default();
        let mut mcus = Vec::with_capacity(config.mcu.len());

        for (name, mcu_config) in config.mcu {
            debug!("Initializing runtime '{name}'");

            let MCUConfig {
                connection,
                connection_wrapper,
                start_delay,
                inner,
            } = mcu_config;

            let stream = init_connection_with_wrapper(connection, connection_wrapper)
                .map_err(|e: MCUError| RuntimeInitError::MCU(e, (&name).into()))?;

            let mcu = match inner {
                MCUConfigType::Sim(sim_mcuconfig) => SimMCURuntime::from_config(sim_mcuconfig)
                    .map_err(|e: MCUError| RuntimeInitError::MCU(e, (&name).into()))?,

                MCUConfigType::Klipper(klipper_mcuconfig) => {
                    KlipperMCURuntime::from_config(stream, klipper_mcuconfig, &mut device_map)
                        .map_err(|e: MCUError| RuntimeInitError::MCU(e, (&name).into()))?
                }
            };

            info!("Initialized runtime '{name}'");

            mcus.push((name, mcu));
        }

        info!("Registered {} MCUs", mcus.len());

        let kinematics = match config.kinematics {
            Kinematics::CoreXY(core_xykinematics) => {
                CoreXYRuntime::from_config(core_xykinematics, device_map, mcus)?
            }
        };

        info!("Initialized printer runtime");

        Ok(Self { kinematics })
    }
}
