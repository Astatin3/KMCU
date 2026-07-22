use crate::{config::sim_mcu::SimMCU, traits::from_config::FromConfig};

pub struct SimMCURuntime {
    // axes:
}

impl FromConfig for SimMCURuntime {
    type ConfigType = SimMCU;

    fn from_config(config: SimMCU) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        todo!()
    }
}
