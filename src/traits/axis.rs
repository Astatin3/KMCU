use crate::{traits::MCU, utils::units};

pub trait Axis {
    fn register_self(&mut self, mcu: &mut dyn MCU)
    where
        Self: Sized;

    fn step(&mut self, mcu: &mut dyn MCU, count: i32, interval: units::ShortTime)
    where
        Self: Sized;
}
