use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    config::{MCUConfig, PrinterConfig, kinematics},
    runtime::{core_xy::CoreXYRuntime, sim_mcu::SimMCURuntime},
    traits::{from_config::FromConfig, mcu::MCU},
};

pub struct PrinterRuntime {
    kinematics: CoreXYRuntime,
}

impl PrinterRuntime {
    // pub fn print(Iter)
}

impl FromConfig for PrinterRuntime {
    type ConfigType = PrinterConfig;

    fn from_config(config: Self::ConfigType) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let mut mcus = HashMap::with_capacity(config.mcu.len());

        for (name, mcu_config) in config.mcu {
            let mcu = match mcu_config {
                MCUConfig::Sim(sim_mcuconfig) => {
                    Rc::new(RefCell::new(SimMCURuntime::from_config(sim_mcuconfig)?))
                        as Rc<RefCell<dyn MCU>>
                }

                _ => todo!(),
            };

            mcus.insert(name, mcu);
        }

        info!("Registered {} MCUs", mcus.len());

        let kinematics = match config.kinematics {
            kinematics::Kinematics::CoreXY(core_xykinematics) => {
                CoreXYRuntime::from_config((core_xykinematics, config.axis, mcus))?
            }
        };

        Ok(Self { kinematics })
    }
}
