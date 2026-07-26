use crate::Res;
use crate::config::{SerialConnection, SocketConnection};
use crate::traits::{Read, Stream, Write};

pub struct Socket {
    config: SocketConnection,
}

impl Socket {
    pub fn new(config: SocketConnection) -> Res<Self> {
        debug!("Dummy Socket: open '{}'", config.path);
        Ok(Self { config })
    }

    pub fn new_serial(config: SerialConnection) -> Res<Self> {
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
    fn read(&mut self, _buf: &mut [u8]) -> Res<usize> {
        Ok(0)
    }
}

impl Write for Socket {
    fn write(&mut self, buf: &[u8]) -> Res<usize> {
        Ok(buf.len())
    }

    fn flush(&mut self) -> Res<()> {
        Ok(())
    }
}

impl Stream for Socket {}
