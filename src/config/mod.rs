// use serde::;

use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    mcu: HashMap<String, MCU>,
}

#[derive(Deserialize)]
pub struct MCU {
    serial: Option<Serial>,
}

#[derive(Deserialize)]
pub struct Serial {
    path: String,
    buad: u32,
}
