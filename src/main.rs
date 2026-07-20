#![allow(unused)]

use std::env::args;

use serialport::SerialPort;

use crate::{
    traits::binary::Binary,
    types::{
        command::{CommandArgFilled, CommandArgOutline, CommandFilled, CommandOutline},
        message::Message,
    },
};

mod types {
    pub mod command;
    pub mod message;
    pub mod serial;
}

mod traits {
    pub mod binary;
}

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
        let command_identify = CommandOutline {
            name: "identify".to_string(),
            id: 1,
            parameters: vec![
                ("offset".to_string(), CommandArgOutline::uint32),
                ("count".to_string(), CommandArgOutline::byte),
            ],
        };

        let mut i = 0;

        loop {
            let byte_start = i * Self::IDENTIFY_COUNT;

            let command = CommandFilled(
                1,
                vec![
                    CommandArgFilled::uint32(byte_start as u32),
                    CommandArgFilled::byte(Self::IDENTIFY_COUNT as u8),
                ],
            );

            self.write(&command);

            let response = self.read()?;

            if response.payload.len() == 0 {
                println!("Empty packet, resending...");
            } else {
                println!("Response: {response:?}");
                i += 1;
            }

            if i > 100 {
                break;
            }
        }

        Ok(())
    }

    fn write(&mut self, command: &CommandFilled) -> anyhow::Result<()> {
        let payload = command.encode();

        let message = Message::new(payload, (self.seq % 16) as u8).unwrap();

        // println!("Sent: {message:?}");

        self.port.write_all(&message.encode())?;

        // self.seq += 1;

        Ok(())
    }

    fn read(&mut self) -> anyhow::Result<Message> {
        let message = Message::decode(&mut self.port, ())?;

        // be sure to increment the sequence number
        self.seq = message.sequence as usize;

        Ok(message)
    }

    pub fn run() {}
}
