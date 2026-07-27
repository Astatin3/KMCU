use crate::config::{SerialConnection, SocketConnection};
use crate::traits::{Read, Stream, Write};
use crate::utils::error::IOError;

pub struct Socket {
    config: SocketConnection,
}

impl Socket {
    pub fn new(config: SocketConnection) -> Result<Self, IOError> {
        debug!("Dummy Socket: open '{}'", config.path);
        Ok(Self { config })
    }

    pub fn new_serial(config: SerialConnection) -> Result<Self, IOError> {
        debug!(
            "Dummy Socket: open serial '{}' at {} baud",
            config.path, config.baud
        );
        Ok(Self {
            config: SocketConnection {
                path: config.path,
                timeout: config.timeout,
            },
        })
    }
}

impl Read for Socket {
    fn read(&mut self, _buf: &mut [u8]) -> Result<usize, IOError> {
        Ok(0)
    }

    fn flush_input(&mut self) -> Result<(), IOError> {
        Ok(())
    }
}

impl Write for Socket {
    fn write(&mut self, buf: &[u8]) -> Result<usize, IOError> {
        Ok(buf.len())
    }

    fn flush_output(&mut self) -> Result<(), IOError> {
        Ok(())
    }
}

impl Stream for Socket {}
