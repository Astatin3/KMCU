use bytes::{BufMut, BytesMut};
use serialport::SerialPort;

use crate::wire::{
    traits::{binary::Binary, connection::Connection},
    types::{
        command::CommandFilled,
        dictionary::DEFAULT_DICT,
        message::{Message, MESSAGE_MAX, MESSAGE_MIN},
    },
};

pub struct Serial {
    port: Box<dyn SerialPort>,
    seq: usize,
}

impl Serial {
    pub fn open(path: &str, baud: u32) -> anyhow::Result<Self> {
        let port = serialport::new(path, baud)
            .timeout(std::time::Duration::from_millis(100))
            .open()?;

        Ok(Self { port, seq: 0 })
    }
}

impl Connection for Serial {
    fn read(&mut self) -> anyhow::Result<Message> {
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

        let payload = &frame[2..length - 3];
        if payload.is_empty() {
            return Ok(Message::Serialized(frame.freeze()));
        }

        let mut cursor = &payload[..];
        let cmd = CommandFilled::decode(&mut cursor, (*DEFAULT_DICT).clone())?;

        Ok(Message::Deserialized(cmd))
    }

    fn write(&mut self, message: &Message) -> anyhow::Result<()> {
        let mut buf = BytesMut::with_capacity(MESSAGE_MAX);
        message.encode(&mut buf);
        self.port.write_all(&buf)?;
        Ok(())
    }

    fn alive_check(&mut self) -> anyhow::Result<()> {
        let bytes = self.port.bytes_to_read()?;
        Ok(())
    }
}
