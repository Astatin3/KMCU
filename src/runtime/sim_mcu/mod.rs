use crate::{
    config::SimMCUConfig,
    error::Res,
    traits::{FromConfig, MCU},
};

pub struct SimMCURuntime {
    // axes: HashMap<String, Box<dyn Axis>>,
}

impl MCU for SimMCURuntime {
    fn alive(&mut self) -> Res<()> {
        Ok(())
    }
}

impl FromConfig for SimMCURuntime {
    type ConfigType = SimMCUConfig;

    fn from_config(_config: SimMCUConfig) -> Res<Self>
    where
        Self: Sized,
    {
        Ok(SimMCURuntime {})
    }
}
