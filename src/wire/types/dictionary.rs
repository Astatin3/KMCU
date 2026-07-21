use std::collections::HashMap;

use super::command::CommandFilled;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArgType {
    Uint32,
    Int32,
    Uint16,
    Int16,
    Byte,
    String,
    ProgmemBuffer,
    Buffer,
}

#[derive(Clone)]
pub struct CommandOutline {
    pub name: String,
    pub id: u16,
    pub parameters: Vec<(String, ArgType)>,
}

/// Represents the format string format for
/// commands sent over by Klipper
#[derive(Clone)]
pub struct Dictionary {
    commands: HashMap<u16, CommandOutline>,
    command_names: HashMap<String, u16>,
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

    /// Creates a CommandFilled from a name and a JSON object of arguments.
    pub fn fill(&self, name: &str, args: serde_json::Value) -> anyhow::Result<CommandFilled> {
        let id = self
            .command_names
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("No such command '{name}'"))?;

        let outline = self.get_outline(*id).unwrap();

        if let Some(obj) = args.as_object() {
            for (param_name, _) in &outline.parameters {
                if !obj.contains_key(param_name.as_str()) {
                    anyhow::bail!("Missing parameter '{param_name}'");
                }
            }
        } else {
            anyhow::bail!("Args must be a JSON object");
        }

        Ok(CommandFilled::new(name, args))
    }
}

impl Dictionary {
    pub fn get_outline(&self, id: u16) -> Option<&CommandOutline> {
        self.commands.get(&id)
    }

    pub fn get_outline_by_name(&self, name: &str) -> Option<&CommandOutline> {
        let id = self.command_names.get(name)?;
        self.commands.get(id)
    }
}

// MessageTypes = {
//     '%u': PT_uint32(), '%i': PT_int32(),
//     '%hu': PT_uint16(), '%hi': PT_int16(),
//     '%c': PT_byte(),
//     '%s': PT_string(), '%.*s': PT_progmem_buffer(), '%*s': PT_buffer(),
// }

impl CommandOutline {
    /// Parse a klipper format descriptor string.
    ///
    /// Supports two formats:
    /// - Named: `"name key=%type key=%type"` (commands, responses)
    /// - Printf: `"text %type text %type"` (output messages, unnamed params auto-named arg0, arg1, ...)
    pub fn from_descriptor(descriptor: &str, id: u16) -> Option<Self> {
        let first_token = descriptor.split(' ').next()?;
        if !first_token.contains('=')
            && let Some(outline) = Self::from_named_descriptor(descriptor, id)
        {
            return Some(outline);
        }
        Self::from_printf_descriptor(descriptor, id)
    }

    /// Named format: `"name key1=%type1 key2=%type2 ..."`
    fn from_named_descriptor(descriptor: &str, id: u16) -> Option<Self> {
        let mut split = descriptor.split(' ');
        let name = split.next()?.to_string();

        let mut parameters = Vec::new();
        for token in split {
            let (param_name, format_str) = token.split_once('=')?;

            let arg_type = match format_str {
                "%u" => ArgType::Uint32,
                "%i" => ArgType::Int32,
                "%hu" => ArgType::Uint16,
                "%hi" => ArgType::Int16,
                "%c" => ArgType::Byte,
                "%s" => ArgType::String,
                "%.*s" => ArgType::ProgmemBuffer,
                "%*s" => ArgType::Buffer,
                _ => return None,
            };

            parameters.push((param_name.to_string(), arg_type));
        }

        Some(Self {
            name,
            id,
            parameters,
        })
    }

    /// Printf format: scan for `%type` specifiers, auto-generate arg0, arg1, ...
    /// The entire descriptor string is used as the name.
    fn from_printf_descriptor(descriptor: &str, id: u16) -> Option<Self> {
        let mut parameters = Vec::new();
        let mut chars = descriptor.chars().peekable();
        let mut arg_idx: u32 = 0;

        while let Some(c) = chars.next() {
            if c != '%' {
                continue;
            }

            let param = match chars.peek() {
                Some('%') => {
                    chars.next();
                    continue;
                }
                Some('*') => {
                    chars.next();
                    if chars.peek() == Some(&'s') {
                        chars.next();
                        Some(ArgType::Buffer)
                    } else {
                        None
                    }
                }
                Some('.') => {
                    chars.next();
                    if chars.peek() == Some(&'*') {
                        chars.next();
                        if chars.peek() == Some(&'s') {
                            chars.next();
                            Some(ArgType::ProgmemBuffer)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                Some('h') => {
                    chars.next();
                    match chars.peek() {
                        Some('u') => {
                            chars.next();
                            Some(ArgType::Uint16)
                        }
                        Some('i') => {
                            chars.next();
                            Some(ArgType::Int16)
                        }
                        _ => None,
                    }
                }
                Some('u') => {
                    chars.next();
                    Some(ArgType::Uint32)
                }
                Some('i') => {
                    chars.next();
                    Some(ArgType::Int32)
                }
                Some('c') => {
                    chars.next();
                    Some(ArgType::Byte)
                }
                Some('s') => {
                    chars.next();
                    Some(ArgType::String)
                }
                _ => None,
            };

            if let Some(arg_type) = param {
                parameters.push((format!("arg{arg_idx}"), arg_type));
                arg_idx += 1;
            }
        }

        if parameters.is_empty() {
            return None;
        }

        Some(Self {
            name: descriptor.to_string(),
            id,
            parameters,
        })
    }
}
