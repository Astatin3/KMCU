use crate::os::GPIO;
use crate::runtime::klipper_mcu::identify::IdentifyResults;
use crate::traits::Stream;

mod axes {
    mod a4988;
    mod tmc2209;

    pub use a4988::KA4988;
    pub use tmc2209::KTMC2209;
}

mod config;
pub mod identify;
mod io;
mod mcu;

pub mod protocol {
    mod command;
    mod dictionary;
    mod message;

    pub use command::{RecvCommand, SendCommand};
    pub use dictionary::{DictionaryRecv, DictionarySend};
    pub use message::{Frame, FramePayload};
}

use alloc::boxed::Box;
use protocol::DictionarySend;

pub struct KlipperMCURuntime {
    pub stream: Box<dyn Stream>,
    pub seq: u8,

    pub identity: IdentifyResults,
}
