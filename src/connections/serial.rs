use std::io::{Read, Write};
use std::time::Duration;

use serialport::SerialPort;

use crate::config::SerialConnection;
use crate::traits::from_config::FromConfig;

pub struct Serial {
    port: Box<dyn SerialPort>,
}

impl Serial {
    pub fn open(path: &str, baud: u32) -> anyhow::Result<Self> {
        Self::with_timeout(path, baud, Duration::from_millis(100))
    }

    pub fn with_timeout(path: &str, baud: u32, timeout: Duration) -> anyhow::Result<Self> {
        let port = serialport::new(path, baud).timeout(timeout).open()?;
        debug!("Opened serial port '{path}', '{baud}'");
        Ok(Self { port })
    }
}

impl Read for Serial {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.port.read(buf)
    }
}

impl Write for Serial {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.port.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.port.flush()
    }
}

impl crate::connections::Stream for Serial {}

impl FromConfig for Serial {
    type ConfigType = SerialConnection;

    fn from_config(config: Self::ConfigType) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Self::open(&config.path, config.baud)
    }
}
