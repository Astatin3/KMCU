use std::io::Read;

use anyhow::anyhow;
use bytes::{BufMut, BytesMut};
use serde_json::Value;

use crate::wire::{
    traits::binary::Binary,
    types::dictionary::{ArgType, CommandOutline, Dictionary},
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
        self.args
            .get(key)
            .map(value_to_bytes)
            .unwrap_or_default()
    }

    pub fn take_buffer(&mut self, key: &str) -> Option<Vec<u8>> {
        let value = self.args.as_object_mut()?.remove(key)?;
        Some(value_to_bytes(&value))
    }
}

impl Binary for CommandFilled {
    type EncodeArg = Dictionary;
    type DecodeArg = Dictionary;

    fn encode(&self, buf: &mut BytesMut, dict: Dictionary) {
        let outline = dict
            .get_outline_by_name(&self.name)
            .expect("Unknown command name");

        super::super::vlq::encode_msgid_to(outline.id, buf);

        for (param_name, arg_type) in &outline.parameters {
            let value = self
                .args
                .get(param_name.as_str())
                .expect("Missing parameter");
            encode_value(value, arg_type, buf);
        }
    }

    fn decode(reader: &mut dyn Read, dict: Dictionary) -> anyhow::Result<Self> {
        let id = super::super::vlq::parse_msgid(reader)?;

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

fn encode_value(value: &Value, arg_type: &ArgType, buf: &mut BytesMut) {
    match arg_type {
        ArgType::Uint32 => {
            super::super::vlq::encode_int_to(value.as_u64().unwrap() as u32, buf);
        }
        ArgType::Int32 => {
            super::super::vlq::encode_int_to(value.as_i64().unwrap() as u32, buf);
        }
        ArgType::Uint16 => {
            super::super::vlq::encode_int_to(value.as_u64().unwrap() as u32, buf);
        }
        ArgType::Int16 => {
            super::super::vlq::encode_int_to(value.as_i64().unwrap() as u32, buf);
        }
        ArgType::Byte => {
            buf.put_u8(value.as_u64().unwrap() as u8);
        }
        ArgType::String => {
            let s = value.as_str().unwrap();
            buf.put_u8(s.len() as u8);
            buf.extend_from_slice(s.as_bytes());
        }
        ArgType::ProgmemBuffer | ArgType::Buffer => {
            let bytes = value_to_bytes(value);
            buf.put_u8(bytes.len() as u8);
            buf.extend_from_slice(&bytes);
        }
    }
}

fn decode_value(reader: &mut dyn Read, arg_type: &ArgType) -> anyhow::Result<Value> {
    match arg_type {
        ArgType::Uint32 => Ok(Value::Number(
            super::super::vlq::parse_int(reader)?.into(),
        )),
        ArgType::Int32 => Ok(Value::Number(
            (super::super::vlq::parse_int(reader)? as i32).into(),
        )),
        ArgType::Uint16 => Ok(Value::Number(
            (super::super::vlq::parse_int(reader)? as u16).into(),
        )),
        ArgType::Int16 => Ok(Value::Number(
            (super::super::vlq::parse_int(reader)? as i16).into(),
        )),
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
