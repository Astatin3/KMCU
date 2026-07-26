use alloc::collections::btree_map::BTreeMap;
use alloc::string::String;
use heapless::Vec;
use miniz_oxide::inflate::{decompress_to_vec_with_limit, decompress_to_vec_zlib};
use serde::Deserialize;

use crate::traits::Read;
use crate::{
    runtime::klipper_mcu::{
        KlipperMCURuntime,
        protocol::{DictionaryRecv, DictionarySend, RecvCommand, SendCommand},
    },
    utils::error::{IOError, MCUError},
};

#[derive(Deserialize)]
pub struct IdentifyResults {
    pub app: String,
    pub version: String,
    pub build_versions: String,
    pub license: String,
    pub config: BTreeMap<String, serde_json::Value>,
    pub enumerations: BTreeMap<String, BTreeMap<String, serde_json::Value>>,

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
            config: BTreeMap::new(),
            enumerations: BTreeMap::new(),

            commands: DictionarySend::default_dict(),
            responses: DictionaryRecv::default_dict(),
        }
    }

    pub fn from_zlib_bytes(zlib_bytes: &[u8]) -> Result<Self, IOError> {
        let decompressed =
            decompress_to_vec_zlib(zlib_bytes).map_err(IOError::ZllibDecode)?;

        let mut s = String::from_utf8(decompressed).map_err(|_| IOError::InvalidUTF8)?;

        debug!("Got klipper string: {s}");

        let results: Self = serde_json::from_str(&s).map_err(|_| IOError::InvalidJSON)?;
        Ok(results)
    }
}

const IDENTIFY_COUNT: usize = 40;

impl KlipperMCURuntime {
    /// Reads the identify table from the MCU, decompresses it, and parses the
    /// JSON to produce `IdentifyResults` (including populated command/response
    /// dictionaries).
    pub fn identify(&mut self) -> Result<IdentifyResults, MCUError> {
        let mut i = 0;
        let mut zlib_bytes = Vec::<_, 4096>::new(); // 4 KB should be enough memory to store the JSON

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

        IdentifyResults::from_zlib_bytes(&zlib_bytes).map_err(MCUError::KlipperConnection)
    }
}
