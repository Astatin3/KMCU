use crate::{traits::MCU, units};

pub trait Axis {
    // fn from_mcu(mcu: Rc<RefCell<dyn MCU>>, config: )

    fn step(&mut self, mcu: &mut dyn MCU, count: i32, interval: units::ShortTime)
    where
        Self: Sized;
}
