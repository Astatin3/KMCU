use alloc::string::String;

use crate::utils::error::IOError;

pub struct GPIO {
    pin_str: String,
    invert: bool,
}

impl GPIO {
    pub fn new(pin_str: &str, invert: bool) -> Result<Self, IOError> {
        debug!("Dummy GPIO: init '{pin_str}'");
        Ok(Self {
            pin_str: pin_str.into(),
            invert,
        })
    }

    pub fn set(&self, value: bool) -> Result<(), IOError> {
        debug!(
            "Dummy GPIO: set '{}' to '{}'",
            self.pin_str,
            value ^ self.invert
        );
        Ok(())
    }
}

impl Drop for GPIO {
    fn drop(&mut self) {
        debug!("Dummy GPIO: dropping '{}'", self.pin_str);
    }
}
