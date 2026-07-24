use std::thread::sleep;

use anyhow::anyhow;

use crate::connections::gpio::GPIO;
use crate::runtime::klipper_mcu::identify::IdentifyResults;
use crate::{
    config::{self, KlipperMCU},
    connections::{Stream, rpmsg, socket::Socket},
    traits::from_config::FromConfig,
};

pub mod identify;
mod io;
mod mcu;

pub mod protocol {
    pub mod command;
    pub mod dictionary;
    pub mod message;
    pub mod vlq;
}

use protocol::dictionary::DictionarySend;

pub struct KlipperMCURuntime {
    pub stream: Box<dyn Stream>,
    pub seq: usize,

    pub power_pin: Option<GPIO>,

    pub identity: IdentifyResults,
}

impl FromConfig for KlipperMCURuntime {
    type ConfigType = config::KlipperMCU;

    fn from_config(config: Self::ConfigType) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        if let Some(command) = config.exec_start {
            match std::process::Command::new("/bin/sh")
                .arg("-c")
                .arg(&command)
                .output()
            {
                Ok(o) => {
                    if o.status.success() {
                        info!("Ran command '{command}'")
                    } else {
                        warn!("Command '{command}' resulted in error {}", o.status);
                    }
                }
                Err(_) => warn!("Command '{command}' failed to run!"),
            }
        }

        let power_pin = if let Some(pin_str) = config.power_pin {
            let gpio = GPIO::new(&pin_str, false)?;
            gpio.set(true);
            Some(gpio)
        } else {
            None
        };

        sleep(config.start_delay);

        let stream: Box<dyn Stream> = match config.connection {
            config::Connection::Serial(conn) | config::Connection::Socket(conn) => Box::new(
                Socket::from_config(conn)
                    .map_err(|e| anyhow!("Failed to create socket connection: {e}"))?,
            ),
            config::Connection::Rpmsg(conn) => Box::new(
                rpmsg::RpmsgEndpoint::from_config(conn)
                    .map_err(|e| anyhow!("Failed to create RPMSG connection: {e}"))?,
            ),
        };

        let mut this = Self {
            stream,
            seq: 0,
            power_pin,
            identity: IdentifyResults::empty(),
        };

        // Run the identify sequence
        this.identity = this
            .identify()
            .map_err(|e| anyhow!("Failed identification: {e}"))?;

        Ok(this)
    }
}
