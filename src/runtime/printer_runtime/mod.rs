use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    config::{Kinematics, MCUConfig, PrinterConfig},
    runtime::{core_xy::CoreXYRuntime, klipper_mcu::KlipperMCURuntime, sim_mcu::SimMCURuntime},
    traits::{from_config::FromConfig, mcu::MCU},
};

pub struct PrinterRuntime {
    kinematics: CoreXYRuntime,
}

impl PrinterRuntime {
    pub fn alive(&self) -> anyhow::Result<()> {
        self.kinematics.alive()
    }
}

impl FromConfig for PrinterRuntime {
    type ConfigType = PrinterConfig;

    fn from_config(config: Self::ConfigType) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let mut mcus = HashMap::with_capacity(config.mcu.len());

        for (name, mcu_config) in config.mcu {
            debug!("Initializing runtime '{name}'");

            let mcu = match mcu_config {
                MCUConfig::Sim(sim_mcuconfig) => {
                    Rc::new(RefCell::new(SimMCURuntime::from_config(sim_mcuconfig)?))
                        as Rc<RefCell<dyn MCU>>
                }
                MCUConfig::Klipper(klipper_mcuconfig) => Rc::new(RefCell::new(
                    KlipperMCURuntime::from_config(klipper_mcuconfig).map_err(|e| {
                        anyhow::anyhow!("Failed to start Klipper MCU '{name}': {e}")
                    })?,
                )) as Rc<RefCell<dyn MCU>>,
            };

            mcus.insert(name, mcu);
        }

        info!("Registered {} MCUs", mcus.len());

        let kinematics = match config.kinematics {
            Kinematics::CoreXY(core_xykinematics) => {
                CoreXYRuntime::from_config((core_xykinematics, config.axis, mcus))?
            }
        };

        info!("Initialized printer runtime");

        Ok(Self { kinematics })
    }
}
