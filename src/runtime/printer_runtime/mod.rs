use core::cell::RefCell;

use alloc::{collections::btree_map::BTreeMap, rc::Rc};

use crate::{
    config::{Kinematics, MCUConfig, PrinterConfig},
    runtime::{core_xy::CoreXYRuntime, dummy::SimMCURuntime, klipper_mcu::KlipperMCURuntime},
    traits::MCU,
    utils::error::{RuntimeError, RuntimeInitError},
};

pub struct PrinterRuntime {
    kinematics: CoreXYRuntime,
}

impl PrinterRuntime {
    pub fn alive(&self) -> Result<(), RuntimeError> {
        self.kinematics.alive()
    }

    pub fn from_config(config: PrinterConfig) -> Result<Self, RuntimeInitError>
    where
        Self: Sized,
    {
        let mut mcus = BTreeMap::new(); // with_capacity(config.mcu.len());

        for (name, mcu_config) in config.mcu {
            debug!("Initializing runtime '{name}'");

            let mcu = match mcu_config {
                MCUConfig::Sim(sim_mcuconfig) => {
                    Rc::new(RefCell::new(SimMCURuntime::from_config(sim_mcuconfig)?))
                        as Rc<RefCell<dyn MCU>>
                }
                MCUConfig::Klipper(klipper_mcuconfig) => Rc::new(RefCell::new(
                    KlipperMCURuntime::from_config(klipper_mcuconfig)
                        .map_err(|e| RuntimeInitError::MCU(e, (&name).into()))?,
                )) as Rc<RefCell<dyn MCU>>,
            };

            info!("Initialized runtime '{name}'");

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
