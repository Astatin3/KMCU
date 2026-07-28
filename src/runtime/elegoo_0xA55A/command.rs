use alloc::vec::Vec;

use crate::{
    traits::{Binary, Read, Write},
    utils::{crc16_ccitt, error::IOError},
};

/// Commands in the Elegoo 0xA55A bootloader protocol.
///
/// Each command is framed as:
///   `[0xA5, 0x5A] [command_byte] [payload_len_be_u16] [payload] [crc_be_u16]`
/// where CRC is CRC-16-CCITT (XMODEM) over the payload only.
#[derive(Debug, Clone)]
pub enum BootloaderCmd {
    /// Host → bootloader: check if the bootloader is alive.
    ///
    /// Contains a 32-bit magic value (e.g. `0x12345678`). The bootloader
    /// replies with [`Pong`] echoing the same value. Also resets the
    /// internal auto-jump timer, keeping the bootloader from jumping to
    /// the application firmware.
    Ping(u32),

    /// Bootloader → host: response to [`Ping`].
    ///
    /// Echoes back the magic value from the `Ping` to confirm the
    /// bootloader is operational.
    Pong(u32),

    /// Host → bootloader: erase a flash sector.
    ///
    /// The payload contains the 32-bit start address of the sector to
    /// erase. On success the bootloader replies with [`EraseAck`] at the
    /// same address.
    Erase(u32),

    /// Bootloader → host: acknowledgement of a [`Erase`] command.
    ///
    /// Confirms the sector at the given address was erased.
    EraseAck(u32),

    /// Host → bootloader: program a chunk of data into flash.
    ///
    /// Fields:
    /// * `offset` — starting address in flash for this chunk.
    /// * `length` — total firmware size being programmed (same in every
    ///   chunk of a multi-chunk transfer).
    /// * `size`   — number of bytes in this chunk (may be shorter than
    ///   `data.len()`; padding bytes in `data` should be ignored).
    /// * `data`   — raw bytes to write (padded to a fixed chunk size
    ///   by the sender).
    ///
    /// On success the bootloader replies with [`ProgramAck`] containing
    /// the same `length` and `offset`.
    Program {
        offset: u32,
        length: u32,
        size: u32,
        data: Vec<u8>,
    },

    /// Bootloader → host: acknowledgement of a [`Program`] chunk.
    ///
    /// Confirms that the chunk at `offset` was written, referencing the
    /// total `length` from the original program command.
    ProgramAck { length: u32, offset: u32 },

    /// Host → bootloader: jump to the main application firmware.
    ///
    /// Fire-and-forget: the bootloader branches to the application entry
    /// point immediately and *does not send a response*. After sending
    /// this command the host must wait for the application to boot before
    /// starting Klipper protocol communication.
    Jump,
}

impl BootloaderCmd {
    fn command_byte(&self) -> u8 {
        match self {
            Self::Ping(_) | Self::Pong(_) => 0x03,
            Self::Erase(_) | Self::EraseAck(_) => 0x00,
            Self::Program { .. } | Self::ProgramAck { .. } => 0x01,
            Self::Jump => 0x02,
        }
    }

    fn payload(&self) -> Vec<u8> {
        match self {
            Self::Ping(v) | Self::Pong(v) => v.to_be_bytes().to_vec(),
            Self::Erase(v) | Self::EraseAck(v) => v.to_be_bytes().to_vec(),
            Self::Program {
                offset,
                length,
                size,
                data,
            } => {
                let mut buf = Vec::with_capacity(12 + data.len());
                buf.extend_from_slice(&offset.to_be_bytes());
                buf.extend_from_slice(&length.to_be_bytes());
                buf.extend_from_slice(&size.to_be_bytes());
                buf.extend_from_slice(data);
                buf
            }
            Self::ProgramAck { length, offset } => {
                let mut buf = Vec::with_capacity(8);
                buf.extend_from_slice(&length.to_be_bytes());
                buf.extend_from_slice(&offset.to_be_bytes());
                buf
            }
            Self::Jump => Vec::new(),
        }
    }

    fn from_payload(cmd: u8, payload: &[u8]) -> Result<Self, IOError> {
        match cmd {
            0x03 => {
                if payload.len() != 4 {
                    return Err(IOError::InvalidHeader {
                        header: 0x0300 | (payload.len() as u16),
                    });
                }
                let mut b = [0u8; 4];
                b.copy_from_slice(payload);
                Ok(Self::Pong(u32::from_be_bytes(b)))
            }
            0x00 => {
                if payload.len() != 4 {
                    return Err(IOError::InvalidHeader {
                        header: 0x0000 | (payload.len() as u16),
                    });
                }
                let mut b = [0u8; 4];
                b.copy_from_slice(payload);
                Ok(Self::EraseAck(u32::from_be_bytes(b)))
            }
            0x01 if payload.len() == 8 => {
                let mut b0 = [0u8; 4];
                let mut b1 = [0u8; 4];
                b0.copy_from_slice(&payload[0..4]);
                b1.copy_from_slice(&payload[4..8]);
                Ok(Self::ProgramAck {
                    length: u32::from_be_bytes(b0),
                    offset: u32::from_be_bytes(b1),
                })
            }
            0x01 if payload.len() >= 12 => {
                let mut b0 = [0u8; 4];
                let mut b1 = [0u8; 4];
                let mut b2 = [0u8; 4];
                b0.copy_from_slice(&payload[0..4]);
                b1.copy_from_slice(&payload[4..8]);
                b2.copy_from_slice(&payload[8..12]);
                let data = payload[12..].to_vec();
                Ok(Self::Program {
                    offset: u32::from_be_bytes(b0),
                    length: u32::from_be_bytes(b1),
                    size: u32::from_be_bytes(b2),
                    data,
                })
            }
            0x02 => {
                if !payload.is_empty() {
                    return Err(IOError::InvalidHeader {
                        header: 0x0200 | (payload.len() as u16),
                    });
                }
                Ok(Self::Jump)
            }
            _ => Err(IOError::UnknownVariant { id: cmd as i16 }),
        }
    }
}

impl Binary for BootloaderCmd {
    type EncodeArg = ();
    type DecodeArg = ();

    fn encode(&self, writer: &mut dyn Write, _arg: &()) -> Result<(), IOError> {
        trace!("Wrote Elegoo 0xA55A command '{self:?}'");

        let payload = self.payload();
        let crc = crc16_ccitt(&payload);

        writer.write_all(&[0xA5, 0x5A])?;
        writer.write_all(&[self.command_byte()])?;
        writer.write_all(&(payload.len() as u16).to_be_bytes())?;
        writer.write_all(&payload)?;
        writer.write_all(&crc)?;
        Ok(())
    }

    fn decode(reader: &mut dyn Read, _arg: &()) -> Result<Self, IOError> {
        scan_for_magic(reader)?;

        let mut cmd_byte = [0u8; 1];
        reader.read_exact(&mut cmd_byte)?;

        let mut len_bytes = [0u8; 2];
        reader.read_exact(&mut len_bytes)?;
        let payload_len = u16::from_be_bytes(len_bytes) as usize;

        let mut payload = Vec::with_capacity(payload_len);
        payload.resize(payload_len, 0);
        if payload_len > 0 {
            reader.read_exact(&mut payload)?;
        }

        let mut crc_bytes = [0u8; 2];
        reader.read_exact(&mut crc_bytes)?;
        let crc_computed = crc16_ccitt(&payload);
        if crc_bytes != crc_computed {
            return Err(IOError::InvalidHeader {
                header: u16::from_be_bytes(crc_bytes),
            });
        }

        let this = Self::from_payload(cmd_byte[0], &payload)?;

        trace!("Read Elegoo 0xA55A command '{this:?}'");

        Ok(this)
    }

    fn size(&self, _arg: &()) -> usize {
        let payload_len = match self {
            Self::Ping(_) | Self::Pong(_) => 4,
            Self::Erase(_) | Self::EraseAck(_) => 4,
            Self::Program { data, .. } => 12 + data.len(),
            Self::ProgramAck { .. } => 8,
            Self::Jump => 0,
        };
        7 + payload_len
    }
}

fn scan_for_magic(reader: &mut dyn Read) -> Result<(), IOError> {
    let mut buf = [0u8; 1];
    loop {
        reader.read_exact(&mut buf)?;
        if buf[0] != 0xA5 {
            continue;
        }
        reader.read_exact(&mut buf)?;
        if buf[0] == 0x5A {
            return Ok(());
        }
    }
}
