use crate::traits::binary::Binary;

// MessageTypes = {
//     '%u': PT_uint32(), '%i': PT_int32(),
//     '%hu': PT_uint16(), '%hi': PT_int16(),
//     '%c': PT_byte(),
//     '%s': PT_string(), '%.*s': PT_progmem_buffer(), '%*s': PT_buffer(),
// }

/// Type that represents one command
pub struct CommandOutline {
    pub name: String,
    pub id: u16,
    pub parameters: Vec<(String, CommandArgOutline)>,
}

pub struct CommandFilled(pub u16, pub Vec<CommandArgFilled>);

impl Binary for CommandFilled {
    type DecodeArg = CommandOutline;

    fn encode(&self) -> Vec<u8> {
        vec![crate::vlq::encode_msgid(self.0), self.1.encode()].encode()
    }

    fn decode(_: &[u8], outline: CommandOutline) -> anyhow::Result<Self> {
        unreachable!()
    }
}

#[allow(non_camel_case_types)]
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

    fn encode(&self) -> Vec<u8> {
        match self {
            CommandArgFilled::uint32(n) => n.to_le_bytes().to_vec(),
            CommandArgFilled::int32(n) => n.to_le_bytes().to_vec(),
            CommandArgFilled::uint16(n) => n.to_le_bytes().to_vec(),
            CommandArgFilled::int16(n) => n.to_le_bytes().to_vec(),
            CommandArgFilled::byte(n) => n.to_le_bytes().to_vec(),
            CommandArgFilled::string(str) => vec![
                crate::vlq::encode_int(str.len() as u32),
                str.clone().into_bytes(),
            ]
            .encode(),
            CommandArgFilled::progmem_buffer(items) => {
                vec![crate::vlq::encode_int(items.len() as u32), items.clone()].encode()
            }
            CommandArgFilled::buffer(items) => {
                vec![crate::vlq::encode_int(items.len() as u32), items.clone()].encode()
            }
        }
    }

    fn decode(_: &[u8], outline: CommandArgOutline) -> anyhow::Result<Self> {
        unreachable!()
    }
}
