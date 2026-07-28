use alloc::boxed::Box;

use crate::{
    config::DummyAxisConfig,
    traits::{Axis, MCU},
    utils::units::ShortTime,
};

pub struct DummyAxis {
    config: DummyAxisConfig,
    position: f32,
}

impl DummyAxis {
    pub fn new(config: DummyAxisConfig) -> Box<dyn Axis> {
        Box::new(Self {
            config,
            position: 0.,
        })
    }
}

impl Axis for DummyAxis {
    fn simple_move(&mut self, velocity: crate::utils::units::Velocity)
    where
        Self: Sized,
    {
    }
}
