use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Connection {
    Serial(SerialConnection),
    // TODO: Support more connection types
}

fn default_baud() -> u32 {
    115_200
}

#[derive(Debug, Deserialize)]
pub struct SerialConnection {
    path: String,
    #[serde(default = "default_baud")]
    baud: u32,
}
