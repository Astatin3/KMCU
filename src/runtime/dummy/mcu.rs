use crate::{config::SimMCUConfig, traits::MCU, utils::error::MCUError};

pub struct SimMCURuntime {
    // axes: HashMap<String, Box<dyn Axis>>,
}

impl MCU for SimMCURuntime {
    fn alive(&mut self) -> Result<(), MCUError> {
        Ok(())
    }
}

impl SimMCURuntime {
    pub fn from_config(_config: SimMCUConfig) -> Result<Self, MCUError>
    where
        Self: Sized,
    {
        Ok(SimMCURuntime {})
    }
}
