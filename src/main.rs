#![allow(unused)]

#[macro_use]
extern crate log;

use crate::{
    config::PrinterConfig, runtime::printer_runtime::PrinterRuntime,
    traits::from_config::FromConfig,
};

#[allow(unused)]
mod config;

mod runtime;

#[allow(unused)]
mod wire;

mod gcode;

mod connections;

mod traits {
    pub mod axis;
    pub mod binary;
    pub mod from_config;
    pub mod mcu;
}

fn main() {
    pretty_env_logger::init();

    // Catch errors
    if let Err(e) = run() {
        log::error!("{}", e);
    }
}

// fn pin_int(pin_name: String) -> u32 {
//     let port = pin_name.chars().nth(1).unwrap() - 'A'; // 'E' - 'A' = 4
//     pin_idx = std::stoi(pin_name.substr(2)); // 17
//     pin = port * 32 + pin_idx; // 4*32 + 17 = 145
// }

fn run() -> anyhow::Result<()> {
    let config = PrinterConfig::parse(include_str!("../kmcu.toml"))?;

    let printer = PrinterRuntime::from_config(config)?;

    printer.alive()?;
    info!("Printer is alive!");

    // let file = args().nth(1).ok_or(anyhow::anyhow!("Must specify file!"))?;

    // let file = File::open(file)?;

    // let gcode = GcodeIter::from_file(file);

    // let _: Vec<()> = gcode
    //     .map(|code| {
    //         println!("Code: {code:?}");
    //     })
    //     .collect();

    Ok(())
}

// #[allow(unused)]
// fn run_mcu() -> anyhow::Result<()> {
//     let device = args()
//         .nth(1)
//         .ok_or(anyhow::anyhow!("Must specify device!"))?;

//     let buad = args().nth(2).ok_or(anyhow::anyhow!("Must specify buad!"))?;

//     let buad = u32::from_str_radix(&buad, 10)?;

//     println!("Opening port...");
//     let serial = wire::connections::serial::Serial::open(&device, buad)?;
//     let mut mcu = runtime::klipper_mcu::KlipperMCURuntime::new(serial)?;

//     Ok(())
// }
