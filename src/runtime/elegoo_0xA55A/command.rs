use alloc::vec::Vec;

use crate::{
    traits::{Binary, Read, Write},
    utils::error::IOError,
};

#[derive(Debug, Clone)]
pub enum BootloaderCmd {
    Ping(u32),
    Pong(u32),
    Erase(u32),
    EraseAck(u32),
    Program {
        offset: u32,
        length: u32,
        size: u32,
        data: Vec<u8>,
    },
    ProgramAck {
        length: u32,
        offset: u32,
    },
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

fn crc16_ccitt(buf: &[u8]) -> [u8; 2] {
    let mut crc: u16 = 0xffff;
    for &byte in buf {
        let mut data = byte as u16;
        data ^= crc & 0xff;
        data ^= (data & 0x0f) << 4;
        crc = ((data << 8) | (crc >> 8)) ^ (data >> 4) ^ (data << 3);
    }
    [(crc >> 8) as u8, (crc & 0xff) as u8]
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
