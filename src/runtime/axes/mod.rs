use crate::{
    config::axis::AxisConfig,
    traits::{axis::Axis, from_config::FromConfig},
};

pub struct DummyAxis;

impl Axis for DummyAxis {
    fn step(&self, direction: bool) {}
}

impl FromConfig for DummyAxis {
    type ConfigType = AxisConfig;

    fn from_config(config: AxisConfig) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self)
    }
}
