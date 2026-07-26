use crate::{
    Res,
    runtime::klipper_mcu::protocol::{
        command::{RecvCommand, SendCommand},
        dictionary::{DictionaryRecv, DictionarySend},
    },
    traits::{Binary, Read, Write},
};

pub const MESSAGE_MIN: usize = 5;
pub const MESSAGE_MAX: usize = 64;
pub const MESSAGE_SYNC: u8 = 0x7e;
pub const MESSAGE_DEST: u8 = 0x10;
pub const MESSAGE_SEQ_MASK: u8 = 0x0f;
pub(crate) const MESSAGE_HEADER_SIZE: usize = 2;
const MESSAGE_TRAILER_SIZE: usize = 3;

pub(crate) fn compose_sequence_number(seq: u8) -> u8 {
    (seq & MESSAGE_SEQ_MASK) | MESSAGE_DEST
}

fn decompose_sequence_number(composed: u8) -> u8 {
    composed & MESSAGE_SEQ_MASK
}

fn crc16_update(mut crc: u16, byte: u8) -> u16 {
    let mut data: u16 = byte as u16;
    data ^= crc & 0xff;
    data ^= (data & 0x0f) << 4;
    crc = ((data << 8) | (crc >> 8)) ^ (data >> 4) ^ (data << 3);
    crc
}

pub(crate) fn crc16_ccitt(buf: &[u8]) -> [u8; 2] {
    let mut crc: u16 = 0xffff;
    for &byte in buf {
        crc = crc16_update(crc, byte);
    }
    [(crc >> 8) as u8, (crc & 0xff) as u8]
}

/// A writer wrapper that computes CRC16-CCITT on the fly as bytes pass through.
pub(crate) struct CrcWriter<'a> {
    inner: &'a mut dyn Write,
    crc: u16,
}

impl<'a> CrcWriter<'a> {
    pub fn new(inner: &'a mut dyn Write) -> Self {
        Self { inner, crc: 0xffff }
    }

    pub fn finish(self) -> (&'a mut dyn Write, [u8; 2]) {
        (self.inner, [(self.crc >> 8) as u8, (self.crc & 0xff) as u8])
    }
}

impl Write for CrcWriter<'_> {
    fn write(&mut self, buf: &[u8]) -> Res<usize> {
        for &byte in buf {
            self.crc = crc16_update(self.crc, byte);
        }
        self.inner.write(buf)
    }

    fn flush(&mut self) -> Res<()> {
        self.inner.flush()
    }
}

pub struct Frame {
    seq: u8,
    pub payload: FramePayload,
}

pub enum FramePayload {
    Empty,
    SendCommand(SendCommand),
    RecvCommand(RecvCommand),
}

impl FramePayload {
    fn size(&self, dict: &DictionarySend) -> usize {
        match self {
            FramePayload::Empty => 0,
            FramePayload::SendCommand(send_command) => send_command.size(dict),
            FramePayload::RecvCommand(recv_command) => recv_command.size(dict),
        }
    }
}

impl Frame {
    pub fn send(seq: u8, cmd: SendCommand) -> Frame {
        Frame {
            seq,
            payload: FramePayload::SendCommand(cmd),
        }
    }

    /// Sequence number from the frame header.
    pub fn seq(&self) -> u8 {
        self.seq
    }

    // /// The payload bytes (between header and trailer).
    // pub fn payload(&self) -> &[u8] {
    //     &self.buf[MESSAGE_HEADER_SIZE..self.len - MESSAGE_TRAILER_SIZE]
    // }

    /// True if the payload is empty (ACK/NAK).
    pub fn is_empty(&self) -> bool {
        if let FramePayload::Empty = self.payload {
            false
        } else {
            true
        }
    }
}

impl Binary for Frame {
    type EncodeArg = DictionarySend;
    type DecodeArg = DictionaryRecv;

    fn encode(&self, writer: &mut dyn Write, dict: &DictionarySend) -> Res<()> {
        if let FramePayload::SendCommand(cmd) = &self.payload {
            let mut writer = CrcWriter::new(writer);

            // Starting size
            let size = self.size(dict) as u8;
            size.encode(&mut writer, &())?;

            // Sequence byte
            let seq = compose_sequence_number(self.seq);
            seq.encode(&mut writer, &())?;

            // The message data
            cmd.encode(&mut writer, dict)?;

            // Finish calculating the CRC
            let (mut writer, crc) = writer.finish();

            // add CRC checksum
            crc[0].encode(writer, &());
            crc[1].encode(writer, &());

            // The ending magic byte
            MESSAGE_SYNC.encode(writer, &());
        } else {
            unreachable!()
        }

        Ok(())
    }

    fn decode(reader: &mut dyn Read, dict: &DictionaryRecv) -> Res<Self> {
        let mut buf = [0u8; MESSAGE_MAX];
        let mut scan_len = 0usize;

        loop {
            if let Some((start, frame_len)) = find_frame(&buf[..scan_len]) {
                if start > 0 {
                    trace!("Discarded {} stale bytes", start);
                }

                let seq = decompose_sequence_number(buf[start + 1]);
                let payload_start = start + MESSAGE_HEADER_SIZE;
                let payload_end = start + frame_len - MESSAGE_TRAILER_SIZE;
                let payload = &buf[payload_start..payload_end];

                if payload.is_empty() {
                    return Ok(Self {
                        seq,
                        payload: FramePayload::Empty,
                    });
                }

                let mut cursor = payload;
                let cmd = RecvCommand::decode(&mut cursor, dict)?;
                trace!("Received command '{cmd:?}'");

                return Ok(Self {
                    seq,
                    payload: FramePayload::RecvCommand(cmd),
                });
            }

            // Try to fill the buffer
            let space = &mut buf[scan_len..];
            match reader.read(space) {
                Ok(0) => return Err(err!("Connection closed")),
                Ok(n) => {
                    scan_len += n;
                }
                Err(e) => return Err(e.into()),
            }

            // Buffer full and no frame found — drop everything before the first sync byte
            if scan_len >= MESSAGE_MAX {
                if let Some(pos) = buf[..scan_len].iter().position(|&b| b == MESSAGE_SYNC) {
                    let keep = scan_len - pos;
                    buf.copy_within(pos..scan_len, 0);
                    scan_len = keep;
                } else {
                    scan_len = 0;
                }
            }
        }
    }

    fn size(&self, dict: &DictionarySend) -> usize {
        self.payload.size(dict) + MESSAGE_MIN
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
