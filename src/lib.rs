// #![no_std]
#![allow(unused)]

#[macro_use]
extern crate log;

#[macro_use]
mod error;

#[allow(unused)]
mod config;

mod runtime;

mod gcode;

mod connections;

mod traits {
    mod axis;
    mod binary;
    mod from_config;
    mod mcu;
    mod stream;

    pub use axis::Axis;
    pub use binary::Binary;
    pub use from_config::FromConfig;
    pub use mcu::MCU;
    pub use stream::{Read, Stream, Write};
}

pub use crate::{
    error::Res,
    config::PrinterConfig, runtime::printer_runtime::PrinterRuntime, traits::FromConfig,
};
