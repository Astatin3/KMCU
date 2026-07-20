use std::io::Read;

use anyhow::anyhow;
use bytes::{BufMut, BytesMut};

use crate::{
    traits::binary::Binary,
    types::dictionary::{CommandOutline, Dictionary},
};

/// Type that represents one command to be serialized
pub struct CommandFilled(pub u16, pub Vec<CommandArgFilled>);

impl Binary for CommandFilled {
    type DecodeArg = Dictionary;

    fn encode(&self, buf: &mut BytesMut) {
        crate::vlq::encode_msgid_to(self.0, buf);
        for arg in &self.1 {
            arg.encode(buf);
        }
    }

    fn decode(reader: &mut dyn Read, dict: Dictionary) -> anyhow::Result<Self> {
        let id = crate::vlq::parse_msgid(reader)?;

        let outline = dict
            .get_outline(id)
            .ok_or(anyhow!("No such command with id '{id}'"))?;

        let mut parsed_params = Vec::new();

        for (_, outline) in &outline.parameters {
            parsed_params.push(CommandArgFilled::decode(reader, *outline)?);
        }

        Ok(Self(id, parsed_params))
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
pub enum CommandArgOutline {
    uint32,
    int32,
    uint16,
    int16,
    byte,
    string,
    progmem_buffer,
    buffer,
}

impl CommandArgOutline {
    pub fn matches(&self, filled: &CommandArgFilled) -> bool {
        match (self, filled) {
            (CommandArgOutline::uint32, CommandArgFilled::uint32(_)) => {}
            (CommandArgOutline::int32, CommandArgFilled::int32(_)) => {}
            (CommandArgOutline::uint16, CommandArgFilled::uint16(_)) => {}
            (CommandArgOutline::int16, CommandArgFilled::int16(_)) => {}
            (CommandArgOutline::byte, CommandArgFilled::byte(_)) => {}
            (CommandArgOutline::string, CommandArgFilled::string(_)) => {}
            (CommandArgOutline::progmem_buffer, CommandArgFilled::progmem_buffer(_)) => {}
            (CommandArgOutline::buffer, CommandArgFilled::buffer(_)) => {}
            _ => return false,
        }
        return true;
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug)]
pub enum CommandArgFilled {
    uint32(u32),
    int32(i32),
    uint16(u16),
    int16(i16),
    byte(u8),
    string(String),
    progmem_buffer(Vec<u8>),
    buffer(Vec<u8>),
}

impl Binary for CommandArgFilled {
    type DecodeArg = CommandArgOutline;

    fn encode(&self, buf: &mut BytesMut) {
        match self {
            CommandArgFilled::uint32(n) => n.encode(buf),
            CommandArgFilled::int32(n) => n.encode(buf),
            CommandArgFilled::uint16(n) => n.encode(buf),
            CommandArgFilled::int16(n) => n.encode(buf),
            CommandArgFilled::byte(n) => n.encode(buf),
            CommandArgFilled::string(s) => {
                buf.put_u8(s.len() as u8);
                buf.extend_from_slice(s.as_bytes());
            }
            CommandArgFilled::progmem_buffer(items) => {
                buf.put_u8(items.len() as u8);
                buf.extend_from_slice(items);
            }
            CommandArgFilled::buffer(items) => {
                buf.put_u8(items.len() as u8);
                buf.extend_from_slice(items);
            }
        }
    }

    fn decode(reader: &mut dyn Read, outline: CommandArgOutline) -> anyhow::Result<Self> {
        match outline {
            CommandArgOutline::uint32 => Ok(Self::uint32(u32::decode(reader, ())?)),
            CommandArgOutline::int32 => Ok(Self::int32(i32::decode(reader, ())?)),
            CommandArgOutline::uint16 => Ok(Self::uint16(u16::decode(reader, ())?)),
            CommandArgOutline::int16 => Ok(Self::int16(i16::decode(reader, ())?)),
            CommandArgOutline::byte => Ok(Self::byte(u8::decode(reader, ())?)),
            CommandArgOutline::string => {
                let mut len_buf = [0u8; 1];
                reader.read_exact(&mut len_buf)?;
                let len = len_buf[0] as usize;
                let mut buf = vec![0u8; len];
                reader.read_exact(&mut buf)?;
                let s = String::from_utf8(buf)?;
                Ok(Self::string(s))
            }
            CommandArgOutline::progmem_buffer => {
                let mut len_buf = [0u8; 1];
                reader.read_exact(&mut len_buf)?;
                let len = len_buf[0] as usize;
                let mut buf = vec![0u8; len];
                reader.read_exact(&mut buf)?;
                Ok(Self::progmem_buffer(buf))
            }
            CommandArgOutline::buffer => {
                let mut len_buf = [0u8; 1];
                reader.read_exact(&mut len_buf)?;
                let len = len_buf[0] as usize;
                let mut buf = vec![0u8; len];
                reader.read_exact(&mut buf)?;
                Ok(Self::buffer(buf))
            }
        }
    }
}
