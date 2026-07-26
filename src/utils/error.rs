use crate::utils::SizedString;

#[derive(Debug, thiserror::Error)]
pub enum MajorStateError {
    #[error("Config -> {0}")]
    Config(#[from] ConfigError),

    #[error("Runtime init -> {0}")]
    RuntimeInit(#[from] RuntimeInitError),

    #[error("Runtime -> {0}")]
    Runtime(#[from] RuntimeError),
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("TOML parse error: {0}")]
    Toml(#[from] toml::de::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum RuntimeInitError {
    #[error("In MCU '{1}' -> {0}")]
    MCU(MCUError, SizedString<16>),
}

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("In MCU '{1}' -> {0}")]
    MCU(MCUError, SizedString<16>),
}

#[derive(Debug, thiserror::Error)]
pub enum MCUError {
    #[error("Klipper connection -> {0}")]
    KlipperConnection(IOError),

    #[error("Klipper protocol -> {0}")]
    KlipperProtocol(IOError),

    #[error("pin '{1}' -> {0}")]
    Pin(IOError, SizedString<4>),
}

#[derive(Debug, thiserror::Error)]
pub enum IOError {
    #[error("Unknown IO error")]
    Unknown,

    #[error("Unexpected null data")]
    UnexpectedNullData,

    #[error("Linux I/O error '{errno}'")]
    Linux { errno: i32 },

    #[error("timeout")]
    Timeout,

    #[error("not connected")]
    NotConnected,

    #[error("invalid pin")]
    InvalidPin,

    #[error("Format error -> {0}")]
    Format(#[from] core::fmt::Error),

    #[error("Connection closed")]
    ConnectionClosed,

    #[error("unknown variant id {id}")]
    UnknownVariant { id: i16 },

    #[error("Zlib decode -> {0}")]
    ZllibDecode(miniz_oxide::inflate::DecompressError),

    #[error("Invalid JSON")]
    InvalidJSON,

    #[error("Invalid UTF-8")]
    InvalidUTF8,

    #[error("unregistered command id {id}")]
    UnregisteredCommand { id: i16 },

    #[error("Failed to fill whole buffer")]
    FailedToFillWholeBuffer,
}

#[cfg(feature = "platform_linux")]
impl From<std::io::Error> for IOError {
    fn from(e: std::io::Error) -> Self {
        IOError::Linux {
            errno: e.raw_os_error().unwrap_or(0),
        }
    }
}

#[cfg(feature = "platform_linux")]
impl From<std::ffi::NulError> for IOError {
    fn from(_: std::ffi::NulError) -> Self {
        IOError::UnexpectedNullData
    }
}

impl From<MCUError> for RuntimeInitError {
    fn from(e: MCUError) -> Self {
        RuntimeInitError::MCU(e, SizedString::new())
    }
}

impl From<RuntimeError> for RuntimeInitError {
    fn from(e: RuntimeError) -> Self {
        match e {
            RuntimeError::MCU(mcu, name) => RuntimeInitError::MCU(mcu, name),
        }
    }
}
