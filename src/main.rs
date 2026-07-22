#![allow(unused)]

#[macro_use]
extern crate log;

use std::{any, env::args};

use serde_json::{Value, json};

use crate::config::PrinterConfig;

mod config;
mod runtime;
mod wire;

fn main() {
    pretty_env_logger::init();

    // Catch errors
    if let Err(e) = run() {
        log::error!("{}", e);
    }
}

fn run() -> anyhow::Result<()> {
    let config = PrinterConfig::parse(include_str!("../kmcu.toml"))?;

    info!("Read config: {config:?}");

    Ok(())
}

fn run_mcu() -> anyhow::Result<()> {
    let device = args()
        .nth(1)
        .ok_or(anyhow::anyhow!("Must specify device!"))?;

    let buad = args().nth(2).ok_or(anyhow::anyhow!("Must specify buad!"))?;

    let buad = u32::from_str_radix(&buad, 10)?;

    println!("Opening port...");
    let serial = wire::connections::serial::Serial::open(&device, buad)?;
    let mut mcu = runtime::klipper_mcu::MCU::new(serial)?;

    Ok(())
}
