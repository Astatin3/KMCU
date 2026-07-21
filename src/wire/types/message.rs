use bytes::{BufMut, Bytes, BytesMut};

use crate::wire::{
    traits::binary::Binary,
    types::{
        command::CommandFilled,
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
    pub fn from_command(command: &CommandFilled, seq: u8, dict: &Dictionary) -> Option<Self> {
        let mut payload = BytesMut::with_capacity(MESSAGE_MAX);
        command.encode(&mut payload, dict.clone());
        Self::new(payload, seq)
    }

    /// Consume the message and return the raw wire bytes.
    pub fn into_bytes(self) -> Bytes {
        match self {
            Self::Serialized(raw) => raw,
            Self::Deserialized { .. } => panic!("into_bytes called on Deserialized message"),
        }
    }

    /// Write the raw wire bytes of a Serialized message to a buffer.
    pub fn encode_to(&self, buf: &mut BytesMut) {
        if let Self::Serialized(raw) = self {
            buf.extend_from_slice(raw);
        }
    }

    /// Wire sequence number from a raw frame
    pub fn wire_seq(frame: &[u8]) -> u8 {
        decompose_sequence_number(frame[1])
    }
}
