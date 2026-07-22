use crate::{
    config::PrinterConfig, runtime::core_xy::CoreXYRuntime, traits::from_config::FromConfig,
};

pub struct PrinterRuntime {
    kinematics: CoreXYRuntime,
}

impl FromConfig for PrinterRuntime {
    type ConfigType = PrinterConfig;

    fn from_config(config: Self::ConfigType) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        // let mcus = vec![
        //     config.sim_mcu.iter().map(|config| Sim)
        // ]

        todo!()
    }
}
