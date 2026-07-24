use std::io::{Read, Write};

use anyhow::anyhow;
use serde_json::Value;

use crate::{
    runtime::klipper_mcu::protocol::{
        dictionary::{ArgType, Dictionary},
        vlq,
    },
    traits::binary::Binary,
};

#[derive(Debug, Clone)]
pub struct CommandFilled {
    pub name: String,
    pub args: Value,
}

impl CommandFilled {
    pub fn new(name: impl Into<String>, args: Value) -> Self {
        Self {
            name: name.into(),
            args,
        }
    }

    pub fn get_buffer(&self, key: &str) -> Vec<u8> {
        self.args.get(key).map(value_to_bytes).unwrap_or_default()
    }

    pub fn take_buffer(&mut self, key: &str) -> Option<Vec<u8>> {
        let value = self.args.as_object_mut()?.remove(key)?;
        Some(value_to_bytes(&value))
    }
}

impl Binary for CommandFilled {
    type EncodeArg = Dictionary;
    type DecodeArg = Dictionary;

    fn encode(&self, writer: &mut dyn Write, dict: Dictionary) {
        let outline = dict
            .get_outline_by_name(&self.name)
            .expect("Unknown command name");

        vlq::encode_msgid_to(outline.id, writer);

        for (param_name, arg_type) in &outline.parameters {
            let value = self
                .args
                .get(param_name.as_str())
                .expect("Missing parameter");
            encode_value(value, arg_type, writer);
        }
    }

    fn decode(reader: &mut dyn Read, dict: Dictionary) -> anyhow::Result<Self> {
        let id = vlq::parse_msgid(reader)?;

        let outline = dict
            .get_outline(id)
            .ok_or(anyhow!("No such command with id '{id}'"))?;

        let mut map = serde_json::Map::new();
        for (param_name, arg_type) in &outline.parameters {
            map.insert(param_name.clone(), decode_value(reader, arg_type)?);
        }

        Ok(Self {
            name: outline.name.clone(),
            args: Value::Object(map),
        })
    }
}

fn encode_value(value: &Value, arg_type: &ArgType, writer: &mut dyn Write) {
    match arg_type {
        ArgType::Uint32 => {
            vlq::encode_int_to(value.as_u64().unwrap() as u32, writer);
        }
        ArgType::Int32 => {
            vlq::encode_int_to(value.as_i64().unwrap() as u32, writer);
        }
        ArgType::Uint16 => {
            vlq::encode_int_to(value.as_u64().unwrap() as u32, writer);
        }
        ArgType::Int16 => {
            vlq::encode_int_to(value.as_i64().unwrap() as u32, writer);
        }
        ArgType::Byte => {
            writer.write_all(&[value.as_u64().unwrap() as u8]).unwrap();
        }
        ArgType::String => {
            let s = value.as_str().unwrap();
            writer.write_all(&[s.len() as u8]).unwrap();
            writer.write_all(s.as_bytes()).unwrap();
        }
        ArgType::ProgmemBuffer | ArgType::Buffer => {
            let bytes = value_to_bytes(value);
            writer.write_all(&[bytes.len() as u8]).unwrap();
            writer.write_all(&bytes).unwrap();
        }
    }
}

fn decode_value(reader: &mut dyn Read, arg_type: &ArgType) -> anyhow::Result<Value> {
    match arg_type {
        ArgType::Uint32 => Ok(Value::Number(vlq::parse_int(reader)?.into())),
        ArgType::Int32 => Ok(Value::Number((vlq::parse_int(reader)? as i32).into())),
        ArgType::Uint16 => Ok(Value::Number((vlq::parse_int(reader)? as u16).into())),
        ArgType::Int16 => Ok(Value::Number((vlq::parse_int(reader)? as i16).into())),
        ArgType::Byte => {
            let mut byte = [0u8; 1];
            reader.read_exact(&mut byte)?;
            Ok(Value::Number(byte[0].into()))
        }
        ArgType::String => {
            let mut len = [0u8; 1];
            reader.read_exact(&mut len)?;
            let mut buf = vec![0u8; len[0] as usize];
            reader.read_exact(&mut buf)?;
            Ok(Value::String(String::from_utf8(buf)?))
        }
        ArgType::ProgmemBuffer | ArgType::Buffer => {
            let mut len = [0u8; 1];
            reader.read_exact(&mut len)?;
            let mut buf = vec![0u8; len[0] as usize];
            reader.read_exact(&mut buf)?;
            Ok(Value::Array(
                buf.into_iter().map(|b| Value::Number(b.into())).collect(),
            ))
        }
    }
}

fn value_to_bytes(value: &Value) -> Vec<u8> {
    match value {
        Value::Array(arr) => arr
            .iter()
            .filter_map(|v| v.as_u64().map(|n| n as u8))
            .collect(),
        Value::String(s) => s.as_bytes().to_vec(),
        _ => vec![],
    }
}
