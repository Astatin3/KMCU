use std::fmt;

use serde::{Deserialize, Deserializer};

#[derive(Debug, Clone)]
pub enum PinTarget {
    Host,
    MCU(String),
}

/// Struct that represents any pin that's passed through
/// ex: <mcu>:<PA15>
#[derive(Debug, Clone)]
pub struct Pin {
    pub inverted: bool,
    pub target: PinTarget,
    pub name: String,
}

impl<'de> Deserialize<'de> for Pin {
    fn deserialize<D>(deserializer: D) -> Result<Pin, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;

        let (inverted, rest) = if let Some(stripped) = raw.strip_prefix('!') {
            (true, stripped)
        } else {
            (false, raw.as_str())
        };

        let (target, name) = if let Some((mcu, name)) = rest.split_once(':') {
            if name.is_empty() {
                return Err(serde::de::Error::custom(format!(
                    "pin name cannot be empty in '{}'",
                    raw
                )));
            }
            let target = if mcu == "host" {
                PinTarget::Host
            } else {
                PinTarget::MCU(mcu.to_string())
            };
            (target, name.to_string())
        } else {
            return Err(serde::de::Error::custom(format!(
                "pin must specify a target MCU in '{}', e.g. 'host:{}'",
                raw, rest
            )));
        };

        Ok(Pin {
            inverted,
            target,
            name,
        })
    }
}

impl fmt::Display for Pin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.inverted {
            write!(f, "!")?;
        }
        match &self.target {
            PinTarget::Host => write!(f, "host:"),
            PinTarget::MCU(name) => write!(f, "{}:", name),
        }?;
        write!(f, "{}", self.name)
    }
}
