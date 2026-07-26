use crate::{Res, config::SimMCUConfig, traits::MCU};

pub struct SimMCURuntime {
    // axes: HashMap<String, Box<dyn Axis>>,
}

impl MCU for SimMCURuntime {
    fn alive(&mut self) -> Res<()> {
        Ok(())
    }
}

impl SimMCURuntime {
    pub fn from_config(_config: SimMCUConfig) -> Res<Self>
    where
        Self: Sized,
    {
        Ok(SimMCURuntime {})
    }
}
