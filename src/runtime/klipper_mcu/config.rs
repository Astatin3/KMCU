use alloc::boxed::Box;

use crate::{
    config::{Connection, ConnectionWrapper, KlipperMCU},
    os::{GPIO, RpmsgEndpoint, Socket, sleep},
    runtime::klipper_mcu::{KlipperMCURuntime, identify::IdentifyResults},
    traits::Stream,
    utils::{
        error::{IOError, MCUError},
        pin::Pin,
    },
};

impl KlipperMCURuntime {
    pub fn from_config(stream: Box<dyn Stream>, config: KlipperMCU) -> Result<Self, MCUError>
    where
        Self: Sized,
    {
        // let power_pin = if let Some(pin_str) = config.power_pin {
        //     let gpio = GPIO::new(
        //         Pin::from_str(&pin_str)
        //             .ok_or(MCUError::Pin(IOError::InvalidPin, (&pin_str).into()))?,
        //         false,
        //     )
        //     .map_err(|e| MCUError::Pin(e, (&pin_str).into()))?;

        //     gpio.set(true).map_err(MCUError::KlipperConnection)?;
        //     Some(gpio)
        // } else {
        //     None
        // };

        let mut this = Self {
            stream,
            seq: 0,
            identity: IdentifyResults::empty(),
        };

        // Run the identify sequence
        this.identity = this.identify()?;

        Ok(this)
    }
}
