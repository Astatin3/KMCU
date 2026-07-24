//! GPIO controller for SYSFS

use std::{thread::sleep, time::Duration};

use anyhow::anyhow;

const GPIO_PREFIX: &str = "/sys/class/gpio";
const DELAY: Duration = Duration::from_millis(10);

fn pin_name_to_int(pin_name: &str) -> Option<u32> {
    let first_byte = pin_name.as_bytes()[1];

    if b'A' > first_byte {
        return None;
    }

    let port = first_byte - b'A';

    let pin_idx: u32 = pin_name[2..].parse().ok()?;

    Some(((port as u32) << 5) + pin_idx)
}

// Equivalent to 'echo <string> > <filename>'
fn write_to_file(path: String, data: &str) -> anyhow::Result<()> {
    std::fs::write(&path, data).map_err(|e| anyhow!("Failed to write '{data}' to '{path}': {e}"))
}

fn set_pin_export(pin_int: u32, export: bool) -> anyhow::Result<()> {
    write_to_file(
        format!(
            "{GPIO_PREFIX}/{}",
            if export { "export" } else { "unexport" }
        ),
        &pin_int.to_string(),
    )
}

fn set_pin_direction(pin_int: u32, direction: bool) -> anyhow::Result<()> {
    write_to_file(
        format!("{GPIO_PREFIX}/gpio{pin_int}/direction"),
        if direction { "in" } else { "out" },
    )
}

fn set_pin_value(pin_int: u32, value: bool) -> anyhow::Result<()> {
    write_to_file(
        format!("{GPIO_PREFIX}/gpio{pin_int}/value"),
        if value { "1" } else { "0" },
    )
}

pub struct GPIO {
    pin_str: String,
    pin_int: u32,

    direction: bool,
    invert: bool,
}

impl GPIO {
    pub fn new(pin_str: &str, direction: bool, invert: bool) -> anyhow::Result<Self> {
        let pin_int = pin_name_to_int(pin_str).ok_or(anyhow!("Invalid GPIO Pin: '{pin_str}'"))?;
        let pin_str = pin_str.to_string();

        // The result is ignored since this might
        // return an error because if already exported
        let _ = set_pin_export(pin_int, true);

        // The kernel takes a sec to set the pin to export
        sleep(DELAY);

        set_pin_direction(pin_int, direction)?;

        set_pin_value(pin_int, invert)?;

        debug!("Initialized GPIO pin '{pin_str}'");

        Ok(Self {
            pin_str,
            pin_int,
            direction,
            invert,
        })
    }

    pub fn set(&self, value: bool) -> anyhow::Result<()> {
        debug!("Set GPIO pin '{}' to '{value}'", self.pin_str);
        set_pin_value(self.pin_int, value ^ self.invert)
    }
}

impl Drop for GPIO {
    fn drop(&mut self) {
        let _ = set_pin_value(self.pin_int, !self.invert);
        let _ = set_pin_export(self.pin_int, false);
    }
}
