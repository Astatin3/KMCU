use std::io::Read;

use bytes::{BufMut, Bytes, BytesMut};

use crate::wire::{
    traits::binary::Binary,
    types::{
        command::{CommandArgFilled, CommandFilled},
        dictionary::Dictionary,
    },
};

pub const MESSAGE_MIN: usize = 5;
pub const MESSAGE_MAX: usize = 64;
pub const MESSAGE_SYNC: u8 = 0x7e;
pub const MESSAGE_DEST: u8 = 0x10;
pub const MESSAGE_SEQ_MASK: u8 = 0x0f;

fn compose_sequence_number(seq: u8) -> u8 {
    (seq & MESSAGE_SEQ_MASK) | MESSAGE_DEST
}

fn decompose_sequence_number(composed: u8) -> u8 {
    composed & MESSAGE_SEQ_MASK
}

fn crc16_ccitt(buf: &[u8]) -> [u8; 2] {
    let mut crc: u16 = 0xffff;
    for &byte in buf {
        let mut data: u16 = byte as u16;
        data ^= crc & 0xff;
        data ^= (data & 0x0f) << 4;
        crc = ((data << 8) | (crc >> 8)) ^ (data >> 4) ^ (data << 3);
    }
    [(crc >> 8) as u8, (crc & 0xff) as u8]
}

#[derive(Debug)]
pub enum Message {
    Serialized(Bytes),
    Deserialized(CommandFilled),
}

impl Message {
    /// Build a Serialized message from a pre-encoded payload and sequence number.
    /// Uses a single BytesMut allocation for the entire frame.
    pub fn new(payload: BytesMut, seq: u8) -> Option<Self> {
        let payload_len = payload.len();
        let length = payload_len + MESSAGE_MIN;
        if length > MESSAGE_MAX {
            return None;
        }

        let seq = seq & MESSAGE_SEQ_MASK;
        let composed = compose_sequence_number(seq);

        let mut frame = BytesMut::with_capacity(length);
        frame.put_u8(length as u8);
        frame.put_u8(composed);
        frame.extend_from_slice(&payload);

        let crc = crc16_ccitt(&frame);
        frame.put_u8(crc[0]);
        frame.put_u8(crc[1]);
        frame.put_u8(MESSAGE_SYNC);

        Some(Self::Serialized(frame.freeze()))
    }

    /// Build a Serialized message directly from a command and sequence number.
    pub fn from_command(command: &CommandFilled, seq: u8) -> Option<Self> {
        let mut payload = BytesMut::with_capacity(MESSAGE_MAX);
        command.encode(&mut payload);
        Self::new(payload, seq)
    }

    /// Consume the message and return the raw wire bytes.
    pub fn into_bytes(self) -> Bytes {
        match self {
            Self::Serialized(raw) => raw,
            Self::Deserialized { .. } => panic!("into_bytes called on Deserialized message"),
        }
    }

    /// Wire sequence number from a raw frame
    pub fn wire_seq(frame: &[u8]) -> u8 {
        decompose_sequence_number(frame[1])
    }
}

impl Binary for Message {
    type DecodeArg = Dictionary;

    fn encode(&self, buf: &mut BytesMut) {
        match self {
            Self::Serialized(raw) => {
                buf.extend_from_slice(raw);
            }
            Self::Deserialized(cmd) => {
                let mut payload = BytesMut::with_capacity(MESSAGE_MAX);
                cmd.encode(&mut payload);

                let payload_len = payload.len();
                let length = payload_len + MESSAGE_MIN;
                let composed = compose_sequence_number(0);

                buf.reserve(length);
                buf.put_u8(length as u8);
                buf.put_u8(composed);
                buf.extend_from_slice(&payload);

                let crc = crc16_ccitt(&buf[buf.len() - payload_len - 2..]);
                buf.put_u8(crc[0]);
                buf.put_u8(crc[1]);
                buf.put_u8(MESSAGE_SYNC);
            }
        }
    }

    fn decode(reader: &mut dyn Read, dict: Dictionary) -> anyhow::Result<Self> {
        let mut len_buf = [0u8; 1];
        reader.read_exact(&mut len_buf)?;
        let length = len_buf[0] as usize;

        if length < MESSAGE_MIN {
            anyhow::bail!("Packet too small: {length}");
        }
        if length > MESSAGE_MAX {
            anyhow::bail!("Packet too large: {length}");
        }

        let mut frame = vec![len_buf[0]; length];
        reader.read_exact(&mut frame[1..])?;

        // Verify sync
        if frame[length - 1] != MESSAGE_SYNC {
            anyhow::bail!("Invalid sync byte");
        }

        // Verify CRC
        let crc_got = [frame[length - 3], frame[length - 2]];
        let crc_calc = crc16_ccitt(&frame[..length - 3]);
        if crc_got != crc_calc {
            anyhow::bail!("CRC mismatch");
        }

        let payload = &frame[2..length - 3];
        if payload.is_empty() {
            anyhow::bail!("ACK");
        }

        let mut cursor = &payload[..];
        let cmd = CommandFilled::decode(&mut cursor, dict)?;
        Ok(Self::Deserialized(cmd))
    }
}
