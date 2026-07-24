use std::io::{Read, Write};
use std::os::unix::process::CommandExt;
use std::process::Command;

use anyhow::anyhow;
use bytes::BytesMut;
use serde_json::json;

use crate::{
    config::{self, KlipperMCU},
    connections::{Stream, rpmsg, serial::Serial, socket::Socket},
    traits::{binary::Binary, from_config::FromConfig, mcu::MCU},
    wire::types::{
        command::CommandFilled,
        dictionary::{DEFAULT_DICT, Dictionary},
        message::Frame,
    },
};

pub mod identify;
mod mcu;

pub struct KlipperMCURuntime {
    stream: Box<dyn Stream>,
    seq: usize,

    commands: Dictionary,
    responses: Dictionary,

    #[allow(dead_code)]
    output: Dictionary,
}

impl KlipperMCURuntime {
    pub fn new(stream: Box<dyn Stream>) -> anyhow::Result<Self> {
        let mut this = Self {
            stream,
            seq: 0,
            commands: DEFAULT_DICT.clone(),
            responses: DEFAULT_DICT.clone(),
            output: DEFAULT_DICT.clone(),
        };

        let results = this.identify()?;

        let (commands, responses, output) = results.build_dictionaries()?;
        this.commands = commands;
        this.responses = responses;
        this.output = output;

        Ok(this)
    }

    fn send_command(&mut self, command: &CommandFilled) -> anyhow::Result<()> {
        let mut payload = BytesMut::with_capacity(64);
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

        let stream: Box<dyn Stream> = match config.connection {
            config::Connection::Serial(conn) => Box::new(
                Serial::from_config(conn)
                    .map_err(|e| anyhow!("Failed to create serial connection: {e}"))?,
            ),
            config::Connection::Socket(conn) => Box::new(
                Socket::from_config(conn)
                    .map_err(|e| anyhow!("Failed to create socket connection: {e}"))?,
            ),
            config::Connection::Rpmsg(conn) => Box::new(
                rpmsg::RpmsgEndpoint::from_config(conn)
                    .map_err(|e| anyhow!("Failed to create RPMSG connection: {e}"))?,
            ),
        };

        Self::new(stream).map_err(|e| anyhow!("Failed startup: {e}"))
    }
}
