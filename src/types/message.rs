use std::io::Read;

use crate::traits::binary::Binary;

/// Message struct according to https://deepwiki.com/Klipper3d/klipper/3.3-communication-protocol

/// Message type used for all communications
#[derive(Debug)]
pub struct Message {
    pub length: usize,
    pub sequence: u8, // Sequence number doesn't have direction bit for readability
    pub payload: Vec<u8>,
    pub crc: [u8; 2],
}

impl Message {
    pub const MESSAGE_MIN: usize = 5; // Min payload size
    pub const MESSAGE_MAX: usize = 64; // Max payload size

    pub const MESSAGE_SYNC: u8 = 0x7e; // The frame sync marker

    pub const MESSAGE_DEST: u8 = 0x10; // Direction bit
    pub const MESSAGE_SEQ_MASK: u8 = 0x0f; // Sequence number mask
}

impl Message {
    /// Creates a new Message packet with some payload and sequence number
    pub fn new(payload: Vec<u8>, seq: u8) -> Option<Self> {
        let length = payload.len() + Self::MESSAGE_MIN;

        // Check the length of the packet
        if length > Self::MESSAGE_MAX {
            return None;
        }

        let sequence = seq & Self::MESSAGE_SEQ_MASK;
        let composed_seq = Self::compose_sequence_number(sequence);
        let mut crc_buf = Vec::with_capacity(2 + payload.len());
        crc_buf.push(length as u8);
        crc_buf.push(composed_seq);
        crc_buf.extend_from_slice(&payload);
        let crc = Self::crc16_ccitt(&crc_buf);

        Some(Self {
            length,
            sequence,
            payload,
            crc,
        })
    }

    fn compose_sequence_number(num: u8) -> u8 {
        (num & Self::MESSAGE_SEQ_MASK) | Self::MESSAGE_DEST
    }

    fn decompose_sequence_number(composed: u8) -> u8 {
        composed & !Self::MESSAGE_DEST
    }

    /// Calculates CRC-16/CCITT checksum
    ///
    /// # Arguments
    /// * `buf` - A slice of bytes to calculate the checksum for.
    ///
    /// # Returns
    /// A vector containing two bytes: [high_byte, low_byte].
    fn crc16_ccitt(buf: &[u8]) -> [u8; 2] {
        let mut crc: u16 = 0xffff;

        for &byte in buf {
            // Cast byte to u16 for calculation
            let mut data: u16 = byte as u16;

            // Perform the XOR operations defined in the Python code
            data ^= crc & 0xff;
            data ^= (data & 0x0f) << 4;

            // Update CRC
            // Note: In Rust, shifts on u16 are safe and won't overflow into larger types automatically
            crc = ((data << 8) | (crc >> 8)) ^ (data >> 4) ^ (data << 3);
        }

        // Return as a vector of two bytes [high, low] to match Python's return list
        [(crc >> 8) as u8, (crc & 0xff) as u8]
    }
}

impl Binary for Message {
    type DecodeArg = ();

    fn encode(&self) -> Vec<u8> {
        // let mut data = Vec::with_capacity(self.length);

        // data[0] = (self.length as u8).to_le();
        // data[1] = Self::compose_sequence_number(self.sequence.to_le());

        // data[2..(self.length - 3)].copy_from_slice(&self.payload);

        // data[(self.length - 3)..(self.length - 1)].copy_from_slice(&self.crc);

        // data[self.length - 1] = Self::MESSAGE_SYNC.to_le();

        // data

        vec![
            (self.length as u8).encode(),
            Self::compose_sequence_number(self.sequence.to_le()).encode(),
            self.payload.clone(),
            self.crc.to_vec(),
            Self::MESSAGE_SYNC.encode(),
        ]
        .encode()
    }

    fn decode(reader: &mut dyn Read, _: ()) -> anyhow::Result<Self> {
        let mut buf: Vec<u8> = Vec::new();

        let mut length_bytes = [0; 1];
        reader.read_exact(&mut length_bytes)?;
        let length = length_bytes[0] as usize;

        if length == 0 {
            anyhow::bail!("Got null packet");
        } else if length < Self::MESSAGE_MIN {
            anyhow::bail!("Packet too small");
        } else if length > Self::MESSAGE_MAX {
            anyhow::bail!("Packet too large");
        }

        // Allocate the fields
        let mut seq_bytes = [0; 1];
        let mut payload_bytes = vec![0; length - Self::MESSAGE_MIN];
        let mut crc_bytes = [0; 2];
        let mut sync_bytes = [0; 1];

        reader.read_exact(&mut seq_bytes)?;
        reader.read_exact(&mut payload_bytes)?;
        reader.read_exact(&mut crc_bytes)?;
        reader.read_exact(&mut sync_bytes)?;

        let seq = Self::decompose_sequence_number(seq_bytes[0]);

        // println!(
        //     "seq: {seq:?}, payload: {payload_bytes:?}, crc: {crc_bytes:?}, crc_actual: {crc_actual:?}, sync: {sync_bytes:?}"
        // );

        // Check sync byte
        if sync_bytes[0] != Self::MESSAGE_SYNC {
            anyhow::bail!("Invalid sync byte");
        }

        let crc_actual =
            Self::crc16_ccitt(&[&length_bytes, &seq_bytes, payload_bytes.as_slice()].concat());

        if crc_actual != crc_bytes {
            anyhow::bail!("Invalid crc checksum");
        }

        Ok(Message {
            length,
            sequence: seq,
            payload: payload_bytes,
            crc: crc_bytes,
        })

        // loop {
        //     while buf.len() >= Self::MESSAGE_MIN {
        //         let candidate_len = buf[0] as usize;

        //         if candidate_len < Self::MESSAGE_MIN || candidate_len > Self::MESSAGE_MAX {
        //             buf.remove(0);
        //             continue;
        //         }

        //         if buf.len() < candidate_len {
        //             break;
        //         }

        //         if buf[candidate_len - 1] != Self::MESSAGE_SYNC {
        //             buf.remove(0);
        //             continue;
        //         }

        //         if (buf[1] & !Self::MESSAGE_SEQ_MASK) != Self::MESSAGE_DEST {
        //             buf.remove(0);
        //             continue;
        //         }

        //         let msg_crc: [u8; 2] = [buf[candidate_len - 3], buf[candidate_len - 2]];
        //         if msg_crc != Self::crc16_ccitt(&buf[..candidate_len - 3]) {
        //             buf.remove(0);
        //             continue;
        //         }

        //         let sequence = Self::decompose_sequence_number(buf[1]);
        //         let payload = buf[2..candidate_len - 3].to_vec();

        //         return Ok(Self {
        //             length: candidate_len,
        //             sequence,
        //             payload,
        //             crc: msg_crc,
        //         });
        //     }

        //     let mut byte = [0u8; 1];
        //     reader.read_exact(&mut byte)?;
        //     buf.push(byte[0]);
        // }
    }
}
