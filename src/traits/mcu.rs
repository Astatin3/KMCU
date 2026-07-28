use crate::utils::error::{IOError, MCUError};

pub trait MCU {
    fn alive(&mut self) -> Result<(), IOError>;
}
