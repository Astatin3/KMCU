use crate::Res;
use crate::config::RpmsgConnection;
use crate::traits::{Read, Stream, Write};

pub struct RpmsgEndpoint {
    config: RpmsgConnection,
}

impl RpmsgEndpoint {
    pub fn from_config(config: RpmsgConnection) -> Res<Self> {
        debug!("Dummy RpmsgEndpoint: connect '{}'", config.channel_name);
        Ok(Self { config })
    }

    pub fn ept_path(&self) -> &str {
        &self.config.channel_name
    }

    pub fn reconnect(&mut self) -> Res<()> {
        debug!("Dummy RpmsgEndpoint: reconnect");
        Ok(())
    }
}

impl Stream for RpmsgEndpoint {}

impl Read for RpmsgEndpoint {
    fn read(&mut self, _buf: &mut [u8]) -> Res<usize> {
        Ok(0)
    }
}

impl Write for RpmsgEndpoint {
    fn write(&mut self, buf: &[u8]) -> Res<usize> {
        Ok(buf.len())
    }

    fn flush(&mut self) -> Res<()> {
        Ok(())
    }
}
