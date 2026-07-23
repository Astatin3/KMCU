use crate::traits::connection::Connection;
use std::collections::HashMap;

use serde::Deserialize;

use crate::{
    runtime::klipper_mcu::KlipperMCU,
    wire::types::{
        command::CommandFilled,
        dictionary::{CommandOutline, Dictionary},
    },
};

#[derive(Debug, Deserialize)]
pub struct IdentifyResults {
    pub app: String,
    pub version: String,
    pub build_versions: String,
    pub license: String,
    pub config: HashMap<String, serde_json::Value>,
    pub enumerations: HashMap<String, HashMap<String, serde_json::Value>>,
    pub commands: HashMap<String, u16>,
    pub responses: HashMap<String, u16>,
    pub output: HashMap<String, u16>,
}

impl IdentifyResults {
    pub fn from_zlib_bytes(zlib_bytes: &[u8]) -> anyhow::Result<Self> {
        let mut z = flate2::read::ZlibDecoder::new(zlib_bytes);
        let mut s = String::new();
        std::io::Read::read_to_string(&mut z, &mut s)?;
        debug!("Got klipper string: {s}");
        let results: Self = serde_json::from_str(&s)?;
        Ok(results)
    }

    pub fn build_dictionaries(&self) -> anyhow::Result<(Dictionary, Dictionary, Dictionary)> {
        let command_outlines = self.build_outlines(&self.commands)?;
        let response_outlines = self.build_outlines(&self.responses)?;
        let output_outlines = self.build_outlines(&self.output)?;

        Ok((
            Dictionary::from_vec_commands(command_outlines),
            Dictionary::from_vec_commands(response_outlines),
            Dictionary::from_vec_commands(output_outlines),
        ))
    }

    fn build_outlines(
        &self,
        messages: &HashMap<String, u16>,
    ) -> anyhow::Result<Vec<CommandOutline>> {
        messages
            .iter()
            .map(|(format, id)| {
                CommandOutline::from_descriptor(format, *id)
                    .ok_or_else(|| anyhow::anyhow!("Invalid command descriptor: {format}"))
            })
            .collect()
    }
}

const IDENTIFY_COUNT: usize = 40;

impl<C: Connection> KlipperMCU<C> {
    /// Reads the identify table from the MCU, decompresses it, and parses the JSON.
    pub fn identify(&mut self) -> anyhow::Result<IdentifyResults> {
        let mut i = 0;
        let mut zlib_bytes = Vec::new();

        loop {
            let byte_start = (i * IDENTIFY_COUNT) as u32;

            self.write(CommandFilled::new(
                "identify",
                serde_json::json!({
                    "offset": byte_start,
                    "count": IDENTIFY_COUNT,
                }),
            ))?;

            // For some reason it sends two packets per, this just drops the
            // ACK packet that's also returned
            let response = self.read()?;
            let _ = self.read()?; // Read the ACK

            if let crate::wire::types::message::Message::Deserialized(mut cmd) = response {
                let buf = cmd.take_buffer("data").unwrap_or_default();
                if buf.is_empty() {
                    break;
                }
                zlib_bytes.extend_from_slice(&buf);
            }

            i += 1;
        }

        IdentifyResults::from_zlib_bytes(&zlib_bytes)
    }
}
