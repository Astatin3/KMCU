use crate::traits::binary::Binary;

/// Message struct according to https://deepwiki.com/Klipper3d/klipper/3.3-communication-protocol

/// Message type used for all communications
pub struct Message {
    length: usize,
    sequence: u8, // Sequence number doesn't have direction bit for readability
    payload: Vec<u8>,
    crc: [u8; 2],
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

        let crc = Self::crc16_ccitt(&payload);

        Some(Self {
            length,
            sequence: seq & Self::MESSAGE_SEQ_MASK,
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

    fn decode(buf: &[u8], _: ()) -> anyhow::Result<Self> {
        if buf.len() < Self::MESSAGE_MIN {
            anyhow::bail!("Buffer too short");
        }

        if buf[buf.len() - 1] != Self::MESSAGE_SYNC {
            anyhow::bail!("Invalid sync marker");
        }

        let length = buf[0] as usize;

        if length != buf.len() {
            anyhow::bail!("Length mismatch");
        }

        if length < Self::MESSAGE_MIN || length > Self::MESSAGE_MAX {
            anyhow::bail!("Invalid length");
        }

        let sequence = Self::decompose_sequence_number(buf[1]);
        let payload = buf[2..(length - 3)].to_vec();
        let crc: [u8; 2] = [buf[length - 3], buf[length - 2]];

        if crc != Self::crc16_ccitt(&payload) {
            anyhow::bail!("CRC mismatch");
        }

        Ok(Self {
            length,
            sequence,
            payload,
            crc,
        })
    }
}
