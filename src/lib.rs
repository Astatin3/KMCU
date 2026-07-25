// #![no_std]
#![allow(unused)]

extern crate alloc;

#[macro_use]
extern crate log;

#[macro_use]
mod error;

mod config;
mod connections;
mod gcode;
mod runtime;
mod vlq;

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
    config::PrinterConfig, error::Res, runtime::printer_runtime::PrinterRuntime, traits::FromConfig,
};
