use crate::utils::error::MCUError;

pub trait MCU {
    fn alive(&mut self) -> Result<(), MCUError>;
}
