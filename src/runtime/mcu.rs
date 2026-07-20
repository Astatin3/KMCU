use std::io::Read;

use anyhow::bail;
use flate2::read::ZlibDecoder;

use crate::wire::{
    traits::connection::Connection,
    types::{
        command::{CommandArgFilled, CommandFilled},
        dictionary::DEFAULT_DICT,
        message::Message,
    },
};

pub struct MCU<C: Connection> {
    connection: C,
    seq: usize,
}

impl<C: Connection> MCU<C> {
    const IDENTIFY_COUNT: usize = 40;

    pub fn new(connection: C) -> anyhow::Result<Self> {
        let mut this = Self { connection, seq: 0 };
        this.identify()?;
        Ok(this)
    }

    /// Reads the identify table
    pub fn identify(&mut self) -> anyhow::Result<()> {
        let mut i = 0;

        let mut zlib_bytes = Vec::new();

        loop {
            let byte_start = (i * Self::IDENTIFY_COUNT) as u32;

            self.write(DEFAULT_DICT.fill(
                "identify",
                vec![
                    CommandArgFilled::uint32(byte_start),
                    CommandArgFilled::byte(Self::IDENTIFY_COUNT as u8),
                ],
            )?)?;

            let mut response = self.read()?;
            let _ = self.read()?; // Read the ACK

            if let Message::Deserialized(mut cmd) = response {
                println!("Response id={}: {:?}", cmd.0, cmd.1);

                if let CommandArgFilled::progmem_buffer(buf) = cmd.1.get_mut(1).unwrap() {
                    if buf.len() == 0 {
                        break;
                    } else {
                        zlib_bytes.append(buf);
                    }
                }
            }

            i += 1;

            // bail!("Could not deserialize!");
        }

        let mut z = ZlibDecoder::new(&zlib_bytes[..]);
        let mut s = String::new();
        z.read_to_string(&mut s);

        println!("Got string: {s}");

        Ok(())
    }

    fn write(&mut self, command: CommandFilled) -> anyhow::Result<()> {
        let message = Message::from_command(&command, (self.seq % 16) as u8)
            .ok_or(anyhow::anyhow!("Message too large"))?;
        self.connection.write(&message)
    }

    fn read(&mut self) -> anyhow::Result<Message> {
        let message = self.connection.read()?;
        if let Message::Serialized(ref raw) = message {
            self.seq = Message::wire_seq(raw) as usize;
        }
        Ok(message)
    }
}
