use crate::config::RpmsgConnection;
use crate::traits::{Read, Stream, Write};
use crate::utils::error::IOError;

pub struct RpmsgEndpoint {
    config: RpmsgConnection,
}

impl RpmsgEndpoint {
    pub fn from_config(config: RpmsgConnection) -> Result<Self, IOError> {
        debug!("Dummy RpmsgEndpoint: connect '{}'", config.channel_name);
        Ok(Self { config })
    }

    pub fn ept_path(&self) -> &str {
        &self.config.channel_name
    }

    pub fn reconnect(&mut self) -> Result<(), IOError> {
        debug!("Dummy RpmsgEndpoint: reconnect");
        Ok(())
    }
}

impl Stream for RpmsgEndpoint {}

impl Read for RpmsgEndpoint {
    fn read(&mut self, _buf: &mut [u8]) -> Result<usize, IOError> {
        Ok(0)
    }

    fn flush_input(&mut self) -> Result<(), IOError> {
        Ok(())
    }
}

impl Write for RpmsgEndpoint {
    fn write(&mut self, buf: &[u8]) -> Result<usize, IOError> {
        Ok(buf.len())
    }

    fn flush_output(&mut self) -> Result<(), IOError> {
        Ok(())
    }
}
