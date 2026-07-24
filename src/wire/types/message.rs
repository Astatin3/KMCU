use std::io::{Read, Write};

use bytes::{BufMut, Bytes, BytesMut};

pub const MESSAGE_MIN: usize = 5;
pub const MESSAGE_MAX: usize = 64;
pub const MESSAGE_SYNC: u8 = 0x7e;
pub const MESSAGE_DEST: u8 = 0x10;
pub const MESSAGE_SEQ_MASK: u8 = 0x0f;
const MESSAGE_HEADER_SIZE: usize = 2;
const MESSAGE_TRAILER_SIZE: usize = 3;

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

#[derive(Debug, Clone)]
pub struct Frame {
    raw: Bytes,
}

impl Frame {
    /// Build a new Frame from a payload and sequence number.
    pub fn new(payload: &[u8], seq: u8) -> Option<Self> {
        let payload_len = payload.len();
        let length = payload_len + MESSAGE_MIN;
        if length > MESSAGE_MAX {
            return None;
        }

        let composed = compose_sequence_number(seq);

        let mut buf = BytesMut::with_capacity(length);
        buf.put_u8(length as u8);
        buf.put_u8(composed);
        buf.extend_from_slice(payload);

        let crc = crc16_ccitt(&buf);
        buf.put_u8(crc[0]);
        buf.put_u8(crc[1]);
        buf.put_u8(MESSAGE_SYNC);

        Some(Self {
            raw: buf.freeze(),
        })
    }

    /// Write the raw frame bytes to a stream.
    pub fn write_to(&self, writer: &mut dyn Write) -> anyhow::Result<()> {
        writer
            .write_all(&self.raw)
            .map_err(|e| anyhow::anyhow!("Failed to write frame: {e}"))
    }

    /// Read a single valid frame from a byte stream.
    ///
    /// Uses sync-anchored validation: scans for `0x7e` sync bytes, then
    /// checks if the length byte and CRC are consistent at that position.
    /// Stale/garbage bytes before the frame are silently discarded.
    pub fn read_from(reader: &mut dyn Read) -> anyhow::Result<Self> {
        let mut buf = Vec::with_capacity(MESSAGE_MAX * 2);
        let mut tmp = [0u8; MESSAGE_MAX];

        loop {
            // Try to find a frame in whatever we have so far.
            if let Some((frame_start, frame_len)) = find_frame(&buf) {
                let frame_bytes = buf[frame_start..frame_start + frame_len].to_vec();
                if frame_start > 0 {
                    trace!("Discarded {} stale bytes", frame_start);
                }
                return Ok(Self {
                    raw: Bytes::from(frame_bytes),
                });
            }

            // No complete frame yet — read more data.
            match reader.read(&mut tmp) {
                Ok(0) => anyhow::bail!("Connection closed"),
                Ok(n) => {
                    buf.extend_from_slice(&tmp[..n]);
                    trace!("read_from: read {} bytes, buf={} bytes", n, buf.len());
                }
                Err(e) => return Err(e.into()),
            }

            // No complete frame yet. If buffer is getting large, trim garbage.
            if buf.len() >= MESSAGE_MAX * 2 {
                let trim = buf.len() - MESSAGE_MAX;
                buf.drain(..trim);
                trace!("Buffer overflow, discarded {} bytes", trim);
            }
        }
    }

    /// Sequence number from the frame header.
    pub fn seq(&self) -> u8 {
        decompose_sequence_number(self.raw[1])
    }

    /// The payload bytes (between header and trailer).
    pub fn payload(&self) -> &[u8] {
        let len = self.raw.len();
        &self.raw[MESSAGE_HEADER_SIZE..len - MESSAGE_TRAILER_SIZE]
    }

    /// True if the payload is empty (ACK/NAK).
    pub fn is_empty(&self) -> bool {
        self.payload().is_empty()
    }

    /// Consume the frame and return the raw wire bytes.
    pub fn into_bytes(self) -> Bytes {
        self.raw
    }
}

/// Scan a buffer for a valid Klipper frame using sync-anchored validation.
///
/// Returns `(start_index, frame_length)` if found, where the frame occupies
/// `buf[start..start+length]`.
fn find_frame(buf: &[u8]) -> Option<(usize, usize)> {
    let len = buf.len();

    for j in 0..len {
        if buf[j] != MESSAGE_SYNC {
            continue;
        }

        // Found a sync byte at position j. Check all valid frame lengths
        // that would place the sync at this position.
        let max_l = (j + 1).min(MESSAGE_MAX);
        for l in (MESSAGE_MIN..=max_l).rev() {
            let i = j + 1 - l;

            // Length self-consistency: buf[i] must equal l
            if buf[i] != l as u8 {
                continue;
            }

            // Verify CRC: computed over buf[i..i+l-3], should match buf[i+l-3..i+l-1]
            let crc_slice = &buf[i..i + l - MESSAGE_TRAILER_SIZE];
            let wire_crc = [buf[i + l - 3], buf[i + l - 2]];
            let computed_crc = crc16_ccitt(crc_slice);

            if wire_crc != computed_crc {
                continue;
            }

            return Some((i, l));
        }
    }

    None
}
