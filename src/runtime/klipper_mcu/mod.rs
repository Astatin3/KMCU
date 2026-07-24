use std::io::{Read, Write};
use std::os::unix::process::CommandExt;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

use anyhow::anyhow;
use serde_json::json;

use crate::connections::gpio::GPIO;
use crate::{
    config::{self, KlipperMCU},
    connections::{Stream, rpmsg, socket::Socket},
    traits::{binary::Binary, from_config::FromConfig, mcu::MCU},
};

pub mod identify;
mod mcu;

pub mod protocol {
    pub mod command;
    pub mod dictionary;
    pub mod message;
    pub mod vlq;
}

use protocol::{
    command::CommandFilled,
    dictionary::{DEFAULT_DICT, Dictionary},
    message::Frame,
};

pub struct KlipperMCURuntime {
    stream: Box<dyn Stream>,
    seq: usize,

    commands: Dictionary,
    responses: Dictionary,

    power_pin: Option<GPIO>,

    #[allow(dead_code)]
    output: Dictionary,
}

impl KlipperMCURuntime {
    fn send_command(&mut self, command: &CommandFilled) -> anyhow::Result<()> {
        let mut payload = Vec::with_capacity(64);
        command.encode(&mut payload, self.commands.clone());

        let seq = (self.seq % 16) as u8;
        let frame = Frame::new(&payload, seq).ok_or_else(|| anyhow!("Message too large"))?;

        trace!("Sent command {command:?}");

        frame
            .write_to(&mut *self.stream)
            .map_err(|e| anyhow!("Failed to send: {e}"))
    }

    fn recv_frame(&mut self) -> anyhow::Result<Frame> {
        let frame = Frame::read_from(&mut *self.stream)?;
        self.seq = frame.seq() as usize;
        Ok(frame)
    }

    fn recv_command(&mut self) -> anyhow::Result<CommandFilled> {
        let frame = self.recv_frame()?;

        if frame.is_empty() {
            anyhow::bail!("Received empty frame (ACK/NAK)");
        }

        let mut cursor = frame.payload();
        let cmd = CommandFilled::decode(&mut cursor, self.responses.clone())?;

        trace!("Received command {cmd:?}");

        Ok(cmd)
    }

    fn recv_frame_or_ack(&mut self) -> anyhow::Result<Option<CommandFilled>> {
        let frame = self.recv_frame()?;

        if frame.is_empty() {
            return Ok(None);
        }

        let mut cursor = frame.payload();
        let cmd = CommandFilled::decode(&mut cursor, self.responses.clone())?;

        trace!("Received command {cmd:?}");

        Ok(Some(cmd))
    }
}

impl FromConfig for KlipperMCURuntime {
    type ConfigType = config::KlipperMCU;

    fn from_config(config: Self::ConfigType) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        if let Some(command) = config.exec_start {
            match Command::new("/bin/sh").arg("-c").arg(&command).output() {
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

        // If there's a power pin configured
        let power_pin = if let Some(pin_str) = config.power_pin {
            let gpio = GPIO::new(&pin_str, false, false)?;
            gpio.set(true);

            Some(gpio)
        } else {
            None
        };

        // Sleep the configured start delay
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
            commands: DEFAULT_DICT.clone(),
            responses: DEFAULT_DICT.clone(),
            output: DEFAULT_DICT.clone(),

            power_pin,
        };

        let results = this
            .identify()
            .map_err(|e| anyhow!("Failed identification: {e}"))?;

        Ok(this)
    }
}
