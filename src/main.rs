#[macro_use]
extern crate log;

use std::env::args;

use crate::config::PrinterConfig;

#[allow(unused)]
mod config;

mod runtime;

#[allow(unused)]
mod wire;

mod traits {
    pub mod binary;
    pub mod connection;
    pub mod from_config;
    pub mod mcu;
}

fn main() {
    pretty_env_logger::init();

    // Catch errors
    if let Err(e) = run_mcu() {
        log::error!("{}", e);
    }
}

fn run() -> anyhow::Result<()> {
    let config = PrinterConfig::parse(include_str!("../kmcu.toml"))?;

    info!("Read config: {config:?}");

    Ok(())
}

#[allow(unused)]
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
