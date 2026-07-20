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

    println!("Opening port...");
    let con = Connection::open(&device)?;

    Ok(())
}

struct Connection {
    port: Box<dyn SerialPort>,
    seq: usize,
}

impl Connection {
    const IDENTIFY_COUNT: usize = 40;

    pub fn open(path: &str) -> anyhow::Result<Self> {
        println!("Opening port {path}...");

        let mut this = Self {
            port: serialport::new(path, 1000000)
                .timeout(std::time::Duration::from_millis(1000))
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

        let identify = CommandFilled(
            1,
            vec![CommandArgFilled::uint32(0), CommandArgFilled::byte(40)],
        );

        let payload = identify.encode();

        let message = Message::new(payload, 0).unwrap().encode();

        println!("Wrote identify command: {message:?}");
        self.port.write_all(&message)?;

        // Read data
        let mut buffer = [0; 20];
        let bytes_read = self.port.read(&mut buffer)?;

        println!("Buffer: {buffer:?}");

        Ok(())
    }

    pub fn run() {}
}
