//! GPIO controller for SYSFS

use std::string::ToString;

use alloc::{format, string::String};

use crate::{Res, utils::pin::Pin};

const GPIO_PREFIX: &str = "/sys/class/gpio";

// Equivalent to 'echo <string> > <filename>'
fn write_to_file(path: String, data: &str) -> Res<()> {
    trace!("Wrote '{data}' to '{path}'");
    std::fs::write(&path, data).map_err(|e| err!("Failed to write '{data}' to '{path}': {e}"))
}

fn set_pin_export(pin_int: u16, export: bool) -> Res<()> {
    write_to_file(
        format!(
            "{GPIO_PREFIX}/{}",
            if export { "export" } else { "unexport" }
        ),
        &pin_int.to_string(),
    )
}

fn set_pin_direction(pin_int: u16, direction: bool) -> Res<()> {
    write_to_file(
        format!("{GPIO_PREFIX}/gpio{pin_int}/direction"),
        if direction { "in" } else { "out" },
    )
}

fn set_pin_value(pin_int: u16, value: bool) -> Res<()> {
    write_to_file(
        format!("{GPIO_PREFIX}/gpio{pin_int}/value"),
        if value { "1" } else { "0" },
    )
}

pub struct GPIO {
    pin: Pin,

    invert: bool,
}

impl GPIO {
    pub fn new(pin: Pin, invert: bool) -> Res<Self> {
        // The result is ignored since this might
        // return an error because if already exported
        let _ = set_pin_export(pin.num, true);

        // Set the value to out
        set_pin_direction(pin.num, false)?;

        // Set to the default value
        set_pin_value(pin.num, invert)?;

        debug!("Initialized GPIO pin '{}'", pin.tag);

        Ok(Self { pin, invert })
    }

    pub fn set(&self, value: bool) -> Res<()> {
        debug!("Set GPIO pin '{}' to '{value}'", self.pin.tag);
        set_pin_value(self.pin.num, value ^ self.invert)
    }
}

impl Drop for GPIO {
    fn drop(&mut self) {
        debug!("Dropping pin '{}'", self.pin.tag);
        let _ = set_pin_value(self.pin.num, self.invert);
        let _ = set_pin_export(self.pin.num, false);
    }
}
