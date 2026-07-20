#![allow(unused)]

use std::env::args;

use bytes::{BufMut, BytesMut};
use serialport::SerialPort;

use crate::{
    traits::binary::Binary,
    types::{
        command::{CommandArgFilled, CommandFilled},
        dictionary::DEFAULT_DICT,
        message::{MESSAGE_MAX, MESSAGE_MIN, Message},
    },
};

mod types {
    pub mod command;
    pub mod dictionary;
    pub mod message;
    pub mod serial;
}

mod traits {
    pub mod binary;
}

mod config;
mod vlq;

fn main() -> anyhow::Result<()> {
    let device = args()
        .nth(1)
        .ok_or(anyhow::anyhow!("Must specify device!"))?;

    let buad = args().nth(2).ok_or(anyhow::anyhow!("Must specify buad!"))?;

    let buad = u32::from_str_radix(&buad, 10)?;

    println!("Opening port...");
    let con = Connection::open(&device, buad)?;

    Ok(())
}

struct Connection {
    port: Box<dyn SerialPort>,
    seq: usize,
}

impl Connection {
    const IDENTIFY_COUNT: usize = 40;

    pub fn open(path: &str, buad: u32) -> anyhow::Result<Self> {
        println!("Opening port {path} at {buad} buad...");

        let mut this = Self {
            port: serialport::new(path, buad)
                .timeout(std::time::Duration::from_millis(100))
                .open()?,
            seq: 0,
        };

        println!("Opened");

        // Init the connection
        this.identify()?;

        Ok(this)
    }

    pub fn identify(&mut self) -> anyhow::Result<()> {
        let mut i = 0;

        loop {
            let byte_start = (i * Self::IDENTIFY_COUNT) as u32;

            self.write(DEFAULT_DICT.fill(
                "identify",
                vec![
                    CommandArgFilled::uint32(byte_start),
                    CommandArgFilled::byte(Self::IDENTIFY_COUNT as u8),
                ],
            )?);

            let response = self.read()?;
            let _ = self.read()?; // Read the ACK

            if let Message::Deserialized { id, args } = &response {
                println!("Response id={id}: {args:?}");
            }

            i += 1;
        }

        Ok(())
    }

    fn write(&mut self, command: CommandFilled) -> anyhow::Result<()> {
        let message = Message::from_command(&command, (self.seq % 16) as u8)
            .ok_or(anyhow::anyhow!("Message too large"))?;

        self.port.write_all(&message.into_bytes())?;

        Ok(())
    }

    fn read(&mut self) -> anyhow::Result<Message> {
        // Read the raw frame to extract the sequence number
        let mut len_buf = [0u8; 1];
        self.port.read_exact(&mut len_buf)?;
        let length = len_buf[0] as usize;

        if length < MESSAGE_MIN || length > MESSAGE_MAX {
            anyhow::bail!("Invalid frame length: {length}");
        }

        let mut frame = BytesMut::with_capacity(length);
        frame.put_u8(len_buf[0]);
        let mut rest = vec![0u8; length - 1];
        self.port.read_exact(&mut rest)?;
        frame.extend_from_slice(&rest);

        self.seq = Message::wire_seq(&frame) as usize;

        // Decode the command from the payload
        let payload = &frame[2..length - 3];
        if payload.is_empty() {
            // ACK — return a Serialized representation of the raw frame
            return Ok(Message::Serialized(frame.freeze()));
        }

        let mut cursor = &payload[..];
        let cmd = CommandFilled::decode(&mut cursor, (*DEFAULT_DICT).clone())?;

        Ok(Message::Deserialized {
            id: cmd.0,
            args: cmd.1,
        })
    }

    pub fn run() {}
}
