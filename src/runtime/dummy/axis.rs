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
    fn step(&mut self, mcu: &mut dyn MCU, count: i32, _interval: ShortTime) {
        self.position += (count as f32) * self.config.step_amount_mm;
    }
}
