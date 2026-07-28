use crate::{traits::MCU, utils::units};

pub trait Axis {
    fn simple_move(&mut self, velocity: units::Velocity);
}
