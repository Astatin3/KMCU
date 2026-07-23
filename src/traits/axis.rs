use std::time::Duration;

pub trait Axis {
    // fn from_mcu(mcu: Rc<RefCell<dyn MCU>>, config: )

    fn step(&mut self, count: i32, interval: Duration)
    where
        Self: Sized;
}
