use std::{any, collections::HashMap};

use super::command::{CommandArgFilled, CommandArgOutline, CommandFilled};

/// Represents the list of commands that are sent over
#[derive(Clone)]
pub struct Dictionary {
    commands: HashMap<u16, CommandOutline>,
    command_names: HashMap<String, u16>,
}

#[derive(Clone)]
pub struct CommandOutline {
    pub name: String,
    pub id: u16,
    pub parameters: Vec<(String, CommandArgOutline)>,
}

/// The default dictionary used for initialization
#[static_init::dynamic]
pub static DEFAULT_DICT: Dictionary = Dictionary::from_vec(vec![
    ("identify_response offset=%u data=%.*s", 0),
    ("identify offset=%u count=%c", 1),
])
.unwrap();

impl Dictionary {
    pub fn from_vec(command_descriptors: Vec<(&str, u16)>) -> Option<Self> {
        let mut commands = Vec::with_capacity(command_descriptors.len());
        for (str, id) in command_descriptors {
            commands.push(CommandOutline::from_descriptor(str, id)?);
        }
        Some(Self::from_vec_commands(commands))
    }

    pub fn from_vec_commands(commands: Vec<CommandOutline>) -> Self {
        let (command_names, commands): (HashMap<String, u16>, HashMap<u16, CommandOutline>) =
            commands
                .into_iter()
                .map(|command| ((command.name.clone(), command.id), (command.id, command)))
                .unzip();

        Self {
            commands,
            command_names,
        }
    }

    /// Converts a command to a CommandFilled
    pub fn fill(&self, name: &str, params: Vec<CommandArgFilled>) -> anyhow::Result<CommandFilled> {
        let id = self
            .command_names
            .get(name)
            .ok_or(anyhow::anyhow!("No such command"))?;

        let outline = self.get_outline(*id).unwrap();

        // Check the parameter length
        if outline.parameters.len() != params.len() {
            anyhow::bail!(
                "Invalid parameter length '{}' expected '{}'",
                params.len(),
                outline.parameters.len()
            );
        }

        // Check the parameter types
        for (i, param) in params.iter().enumerate() {
            let (name_outline, param_outline) = &outline.parameters[i];

            if !param_outline.matches(param) {
                anyhow::bail!("Invalid parameter type for parameter '{name_outline}'");
            }
        }

        Ok(CommandFilled(*id, params))
    }
}

impl Dictionary {
    pub fn get_outline(&self, id: u16) -> Option<&CommandOutline> {
        self.commands.get(&id)
    }
}

impl CommandOutline {
    // MessageTypes = {
    //     '%u': PT_uint32(), '%i': PT_int32(),
    //     '%hu': PT_uint16(), '%hi': PT_int16(),
    //     '%c': PT_byte(),
    //     '%s': PT_string(), '%.*s': PT_progmem_buffer(), '%*s': PT_buffer(),
    // }

    // Convert from C-style parameter string like
    // identify offset=%u count=%c
    pub fn from_descriptor(command_descriptor: &str, id: u16) -> Option<Self> {
        // Get the parameters
        let mut split = command_descriptor.split(" ").into_iter();

        // First split is always the parameter string
        let name = split.next().unwrap().to_string();

        let mut parameters = Vec::new();

        for str in split {
            // Each side of the '=' of each parameter
            let (name, format_str) = str.split_once('=')?;

            let arg_type = match format_str {
                "%u" => CommandArgOutline::uint32,
                "%i" => CommandArgOutline::int32,
                "%hu" => CommandArgOutline::uint16,
                "%hi" => CommandArgOutline::int16,
                "%c" => CommandArgOutline::byte,
                "%s" => CommandArgOutline::string,
                "%.*s" => CommandArgOutline::progmem_buffer,
                "%*s" => CommandArgOutline::buffer,
                _ => return None, // Unrecognized parameter
            };

            parameters.push((name.to_string(), arg_type));
        }

        Some(Self {
            name,
            id,
            parameters,
        })
    }
}
