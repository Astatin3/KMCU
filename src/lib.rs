#![no_std]
#![allow(nonstandard_style)]
#![allow(unused)]

// In case we need STD for platform-specific purposes
#[cfg(feature = "std")]
extern crate std;

// We ALWAYS need alloc for klipper
extern crate alloc;

// Add the log macros everywhere
#[macro_use]
extern crate log;

// The UOM units macros require this
#[macro_use]
extern crate uom;

// Add the err! macro everywhere mimicking anyhow::anyhow!
#[macro_use]
mod utils;

mod config;
mod gcode;
mod os;
mod runtime;

mod traits {
    mod axis;
    mod binary;
    mod from_config;
    mod mcu;
    mod stream;

    pub use axis::Axis;
    pub use binary::Binary;
    pub use mcu::MCU;
    pub use stream::{Read, Stream, Write};
}

pub use crate::{
    config::PrinterConfig, runtime::printer_runtime::PrinterRuntime, utils::error::Res,
};
