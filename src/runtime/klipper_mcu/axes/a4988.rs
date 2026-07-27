use crate::traits::Axis;

pub struct KA4988 {}

impl Axis for KA4988 {
    fn register_self(&mut self, mcu: &mut dyn crate::traits::MCU)
    where
        Self: Sized,
    {
        todo!()
    }

    fn step(
        &mut self,
        mcu: &mut dyn crate::traits::MCU,
        count: i32,
        interval: crate::utils::units::ShortTime,
    ) where
        Self: Sized,
    {
        todo!()
    }
}
