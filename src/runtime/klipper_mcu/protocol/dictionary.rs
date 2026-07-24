use std::collections::HashMap;

use serde::{Deserialize, Deserializer, de::Error};

use crate::runtime::klipper_mcu::protocol::command::{RecvCommand, SendCommand};

pub const MAX_DYNAMIC_ID: usize = 173;
pub const MAX_STATIC_ID: usize = 108;

const DYNAMIC_ID_OFFSET: i16 = 32;

/// A map from static ids of commands (the local ones that don't change) to
/// the ids returned by the Klipper MCU
#[derive(Clone)]
pub struct Dictionary {
    // A map from the static id range to dynamic ids
    from_static_id: [i16; MAX_DYNAMIC_ID],

    // A map from the dynamic id range, to static ids
    // indices are offset by DYNAMIC_ID_OFFSET in order to
    // account for negative ids.
    to_static_id: [u16; MAX_STATIC_ID],
}

impl Dictionary {
    pub fn new() -> Self {
        Self {
            from_static_id: [i16::MAX; MAX_DYNAMIC_ID],
            to_static_id: [u16::MAX; MAX_STATIC_ID],
        }
    }

    /// Returns a list of the default commands
    pub fn default_dict() -> Self {
        let mut dict = Dictionary::new();

        // The default commands
        dict.add_definition(RecvCommand::id_for_name("identify_response"), 0);
        dict.add_definition(SendCommand::id_for_name("identify"), 1);

        dict
    }

    pub fn get_static_id(&self, dynamic_id: i16) -> Option<u16> {
        // Offset the dynamic id so it fits within a smaller range
        let dynamic_id_index = (dynamic_id + DYNAMIC_ID_OFFSET) as usize;

        if dynamic_id_index >= MAX_DYNAMIC_ID {
            return None;
        }

        let static_id = self.to_static_id[dynamic_id_index];

        (static_id != u16::MAX).then_some(static_id)
    }

    pub fn get_dynamic_id(&self, static_id: u16) -> Option<i16> {
        let static_id = static_id as usize;

        if static_id >= MAX_STATIC_ID {
            return None;
        }

        let dynamic_id = self.from_static_id[static_id];

        (dynamic_id != i16::MAX).then_some(dynamic_id)
    }

    pub fn add_definition(&mut self, static_id: u16, dynamic_id: i16) {
        if static_id as usize >= MAX_DYNAMIC_ID {
            unreachable!()
        }

        self.from_static_id[static_id as usize] = dynamic_id;
        self.to_static_id[(dynamic_id + DYNAMIC_ID_OFFSET) as usize] = static_id;
    }

    fn deserialize_with<'de, De: Deserializer<'de>>(
        deserializer: De,
        id_for_name: &dyn Fn(&str) -> u16,
    ) -> Result<Dictionary, De::Error> {
        let mut dict = Dictionary::new();

        let map: HashMap<String, i16> = HashMap::deserialize(deserializer)?;
        for (fmt, dynamic_id) in &map {
            let name = fmt.split_whitespace().next().unwrap_or(fmt);

            let static_id = id_for_name(name);

            if static_id == u16::MAX {
                // return Err(Error::custom(format!("Unknown command name '{name}'")));
                warn!("Received unsupported command name '{name}'");

                continue;

                // return Err(Error::custom(format!("Unknown command name '{name}'")));
            }

            dict.add_definition(static_id, *dynamic_id);
        }
        Ok(dict)
    }

    pub fn deserialize_send_command<'de, De: Deserializer<'de>>(
        deserializer: De,
    ) -> Result<Dictionary, De::Error> {
        Self::deserialize_with(deserializer, &SendCommand::id_for_name)
    }

    pub fn deserialize_recv_command<'de, De: Deserializer<'de>>(
        deserializer: De,
    ) -> Result<Dictionary, De::Error> {
        Self::deserialize_with(deserializer, &RecvCommand::id_for_name)
    }
}
