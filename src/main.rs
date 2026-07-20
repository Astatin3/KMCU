#![allow(unused)]

use std::env::args;

mod config;
mod runtime;
mod wire;

fn main() -> anyhow::Result<()> {
    let device = args()
        .nth(1)
        .ok_or(anyhow::anyhow!("Must specify device!"))?;

    let buad = args().nth(2).ok_or(anyhow::anyhow!("Must specify buad!"))?;

    let buad = u32::from_str_radix(&buad, 10)?;

    println!("Opening port...");
    let serial = wire::connections::serial::Serial::open(&device, buad)?;
    let mut mcu = runtime::mcu::MCU::new(serial)?;

    Ok(())
}
