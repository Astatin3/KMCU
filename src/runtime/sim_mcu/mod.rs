use crate::{
    config::sim_mcu::SimMCUConfig,
    traits::{from_config::FromConfig, mcu::MCU},
};

pub struct SimMCURuntime {
    // axes: HashMap<String, Box<dyn Axis>>,
}

impl MCU for SimMCURuntime {
    fn alive(&mut self) -> anyhow::Result<bool> {
        Ok(true)
    }
}

impl FromConfig for SimMCURuntime {
    type ConfigType = SimMCUConfig;

    fn from_config(_config: SimMCUConfig) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(SimMCURuntime {})
    }
}
