use std::collections::HashMap;

use serde::Deserialize;

use crate::error::Res;
use crate::runtime::klipper_mcu::{
    KlipperMCURuntime,
    protocol::{
        command::{RecvCommand, SendCommand},
        dictionary::{DictionaryRecv, DictionarySend},
    },
};

#[derive(Deserialize)]
pub struct IdentifyResults {
    pub app: String,
    pub version: String,
    pub build_versions: String,
    pub license: String,
    pub config: HashMap<String, serde_json::Value>,
    pub enumerations: HashMap<String, HashMap<String, serde_json::Value>>,

    pub commands: DictionarySend,
    pub responses: DictionaryRecv,
}

impl IdentifyResults {
    pub fn empty() -> Self {
        Self {
            app: String::new(),
            version: String::new(),
            build_versions: String::new(),
            license: String::new(),
            config: HashMap::new(),
            enumerations: HashMap::new(),

            commands: DictionarySend::default_dict(),
            responses: DictionaryRecv::default_dict(),
        }
    }

    pub fn from_zlib_bytes(zlib_bytes: &[u8]) -> Res<Self> {
        let mut z = flate2::read::ZlibDecoder::new(zlib_bytes);
        let mut s = String::new();
        std::io::Read::read_to_string(&mut z, &mut s)
            .map_err(|e| err!("Failed to read zlib data: {e}"))?;

        debug!("Got klipper string: {s}");

        let results: Self = serde_json::from_str(&s)
            .map_err(|e| err!("Failed to parse identify JSON: {e}"))?;
        Ok(results)
    }
}

const IDENTIFY_COUNT: usize = 40;

impl KlipperMCURuntime {
    /// Reads the identify table from the MCU, decompresses it, and parses the
    /// JSON to produce `IdentifyResults` (including populated command/response
    /// dictionaries).
    pub fn identify(&mut self) -> Res<IdentifyResults> {
        let mut i = 0;
        let mut zlib_bytes = Vec::new();

        loop {
            let byte_start = (i * IDENTIFY_COUNT) as u32;

            self.send_command(SendCommand::identify {
                offset: byte_start,
                count: IDENTIFY_COUNT as u8,
            })?;

            let cmd = self.recv_frame_or_ack()?;

            match cmd {
                Some(RecvCommand::identify_response { offset, data }) => {
                    // If the MCU returned no new data, assume that's the end
                    if data.is_empty() {
                        break;
                    }

                    zlib_bytes.extend_from_slice(&data);
                    i += 1;
                }

                // the command isn't what is expected, skip it
                _ => {
                    trace!("Skipped command");
                    continue;
                }
            };
        }

        IdentifyResults::from_zlib_bytes(&zlib_bytes)
    }
}
