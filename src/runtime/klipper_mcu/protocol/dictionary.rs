use std::collections::HashMap;

use serde::{Deserialize, Deserializer, de::Error};

use crate::runtime::klipper_mcu::protocol::command::{RecvCommand, SendCommand};

/// Offset applied to dynamic command IDs to handle negative values.
///
/// Klipper's MCU protocol allows dynamic IDs to be negative (e.g., for
/// certain response types). This offset shifts them into a non-negative
/// range so they can be used as array indices.
const DYNAMIC_ID_OFFSET: i16 = 32;

/// Maximum number of static send command IDs.
///
/// This corresponds to the total number of send commands defined in the
/// `SendCommand` enum. Used as the upper bound for valid static IDs
/// when registering command definitions.
pub const MAX_SEND_STATIC_ID: usize = 107;

/// Maximum number of dynamic receive command IDs.
///
/// Klipper's MCU can assign dynamic IDs up to 150 for receive commands.
/// Combined with `DYNAMIC_ID_OFFSET`, this determines the size of
/// internal lookup arrays.
pub const MAX_RECV_DYNAMIC_ID: usize = 150 + DYNAMIC_ID_OFFSET as usize;

/// Mapping from static send command IDs to their dynamic counterparts.
///
/// When encoding commands to send to the MCU, each command has a fixed
/// "static" ID (compile-time, based on enum variant order). The MCU
/// assigns different "dynamic" IDs during initialization. This dictionary
/// translates between them for outbound messages.
///
/// # Example
///
/// ```text
/// Static ID 3 ("gcode") might map to dynamic ID 7 on the MCU.
/// When sending a gcode command, we look up ID 3 → 7 and write 7 on the wire.
/// ```
#[derive(Clone)]
pub struct DictionarySend {
    // A map from the static id range to dynamic ids.
    // Unmapped entries are set to `i16::MAX`.
    from_static_id: [i16; MAX_SEND_STATIC_ID],
}

impl DictionarySend {
    /// Creates an empty dictionary with no command mappings.
    pub fn new() -> Self {
        Self {
            from_static_id: [i16::MAX; MAX_SEND_STATIC_ID],
        }
    }

    /// Creates a dictionary pre-populated with the default commands.
    ///
    /// The default set includes only the `identify` command, which is
    /// always available before full MCU initialization.
    pub fn default_dict() -> Self {
        let mut dict = DictionarySend::new();
        dict.add_definition(SendCommand::id_for_name("identify"), 1);
        dict
    }

    /// Looks up the dynamic ID for a given static command ID.
    ///
    /// Returns `None` if the static ID is out of bounds or not registered.
    pub fn get_dynamic_id(&self, static_id: u8) -> Option<i16> {
        let static_id = static_id as usize;

        if static_id >= MAX_SEND_STATIC_ID {
            return None;
        }

        let dynamic_id = self.from_static_id[static_id];

        (dynamic_id != i16::MAX).then_some(dynamic_id)
    }

    /// Registers a mapping from a static command ID to a dynamic ID.
    ///
    /// # Panics
    ///
    /// Panics if `static_id >= MAX_SEND_STATIC_ID` (should never happen
    /// with valid command definitions).
    pub fn add_definition(&mut self, static_id: u8, dynamic_id: i16) {
        if static_id as usize >= MAX_SEND_STATIC_ID {
            unreachable!()
        }

        self.from_static_id[static_id as usize] = dynamic_id;
    }
}

/// Deserializes a [`DictionarySend`] from a JSON object.
///
/// The expected format is a JSON object where keys are command names
/// (possibly with extra text after whitespace) and values are their
/// dynamic IDs. For example:
///
/// ```json
/// { "identify 1.0": 1, "gcode 1.0": 7, "emergency_stop 1.0": 2 }
/// ```
///
/// Only the first whitespace-delimited token of each key is used as
/// the command name. Unknown command names are logged and skipped.
impl<'de> Deserialize<'de> for DictionarySend {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let mut dict = DictionarySend::new();

        let map: HashMap<String, i16> = HashMap::deserialize(deserializer)?;
        for (fmt, dynamic_id) in &map {
            let name = fmt.split_whitespace().next().unwrap_or(fmt);

            let static_id = SendCommand::id_for_name(name);

            if static_id == u8::MAX {
                warn!("Received unsupported command name '{name}'");
                continue;
            }

            dict.add_definition(static_id, *dynamic_id);
        }
        Ok(dict)
    }
}

/// Mapping from dynamic receive command IDs to their static counterparts.
///
/// When decoding messages received from the MCU, each message begins with
/// a dynamic ID assigned by the MCU. This dictionary translates that
/// dynamic ID back to the fixed "static" ID used internally, allowing
/// the correct `RecvCommand` variant to be constructed.
///
/// # Example
///
/// ```text
/// MCU sends dynamic ID -5 for a temperature response.
/// We look up -5 → static ID 12, then match against RecvCommand::variant(12).
/// ```
#[derive(Clone)]
pub struct DictionaryRecv {
    // A map from the dynamic id range to static ids.
    // Indices are offset by DYNAMIC_ID_OFFSET to handle negative dynamic IDs.
    // Unmapped entries are set to `u8::MAX`.
    to_static_id: [u8; MAX_RECV_DYNAMIC_ID],
}

impl DictionaryRecv {
    /// Creates an empty dictionary with no command mappings.
    pub fn new() -> Self {
        Self {
            to_static_id: [u8::MAX; MAX_RECV_DYNAMIC_ID],
        }
    }

    /// Creates a dictionary pre-populated with the default commands.
    ///
    /// The default set includes only the `identify_response` command,
    /// which is always available before full MCU initialization.
    pub fn default_dict() -> Self {
        let mut dict = DictionaryRecv::new();
        dict.add_definition(RecvCommand::id_for_name("identify_response"), 0);
        dict
    }

    /// Looks up the static ID for a given dynamic command ID.
    ///
    /// The dynamic ID is offset by `DYNAMIC_ID_OFFSET` before indexing.
    /// Returns `None` if the resulting index is out of bounds or not
    /// registered.
    pub fn get_static_id(&self, dynamic_id: i16) -> Option<u8> {
        let dynamic_id_index = (dynamic_id + DYNAMIC_ID_OFFSET) as usize;

        if dynamic_id_index >= MAX_SEND_STATIC_ID {
            return None;
        }

        let static_id = self.to_static_id[dynamic_id_index];

        (static_id != u8::MAX).then_some(static_id)
    }

    /// Registers a mapping from a static command ID to a dynamic ID.
    ///
    /// The dynamic ID is offset by `DYNAMIC_ID_OFFSET` before indexing.
    ///
    /// # Panics
    ///
    /// Panics if `static_id >= MAX_SEND_STATIC_ID` (should never happen
    /// with valid command definitions).
    pub fn add_definition(&mut self, static_id: u8, dynamic_id: i16) {
        if static_id as usize >= MAX_SEND_STATIC_ID {
            unreachable!()
        }

        self.to_static_id[(dynamic_id + DYNAMIC_ID_OFFSET) as usize] = static_id;
    }
}

/// Deserializes a [`DictionaryRecv`] from a JSON object.
///
/// The expected format is a JSON object where keys are command names
/// (possibly with extra text after whitespace) and values are their
/// dynamic IDs. For example:
///
/// ```json
/// { "identify_response 1.0": 0, "last_response 1.0": -5 }
/// ```
///
/// Only the first whitespace-delimited token of each key is used as
/// the command name. Unknown command names are logged and skipped.
impl<'de> Deserialize<'de> for DictionaryRecv {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let mut dict = DictionaryRecv::new();

        let map: HashMap<String, i16> = HashMap::deserialize(deserializer)?;
        for (fmt, dynamic_id) in &map {
            let name = fmt.split_whitespace().next().unwrap_or(fmt);

            let static_id = RecvCommand::id_for_name(name);

            if static_id == u8::MAX {
                warn!("Received unsupported command name '{name}'");
                continue;
            }

            dict.add_definition(static_id, *dynamic_id);
        }
        Ok(dict)
    }
}
