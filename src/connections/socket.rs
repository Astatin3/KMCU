use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::io::{AsRawFd, RawFd};
use std::time::Duration;

use crate::config::SocketConnection;
use crate::traits::from_config::FromConfig;

pub struct Socket {
    file: File,
    timeout: Duration,
}

impl Socket {
    pub fn open(path: &str) -> anyhow::Result<Self> {
        Self::with_timeout(path, Duration::from_secs(5))
    }

    pub fn with_timeout(path: &str, timeout: Duration) -> anyhow::Result<Self> {
        let file = File::options().read(true).write(true).open(path)?;
        Ok(Self { file, timeout })
    }

    fn poll_ready(&self, fd: RawFd, events: i16) -> anyhow::Result<()> {
        let mut pollfd = libc::pollfd {
            fd,
            events,
            revents: 0,
        };

        let ms = self.timeout.as_millis() as i32;
        let ret = unsafe { libc::poll(&mut pollfd, 1, ms) };

        if ret < 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        if ret == 0 {
            anyhow::bail!("Timed out after {:?}", self.timeout);
        }
        if pollfd.revents & libc::POLLERR != 0 {
            anyhow::bail!("Poll error on socket device");
        }

        Ok(())
    }
}

impl Read for Socket {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.poll_ready(self.file.as_raw_fd(), libc::POLLIN)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::TimedOut, e.to_string()))?;
        self.file.read(buf)
    }
}

impl Write for Socket {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.poll_ready(self.file.as_raw_fd(), libc::POLLOUT)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::TimedOut, e.to_string()))?;
        self.file.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()
    }
}

impl crate::connections::Stream for Socket {
    fn read_timeout(&self) -> Duration {
        self.timeout
    }
}

impl FromConfig for Socket {
    type ConfigType = SocketConnection;

    fn from_config(config: Self::ConfigType) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Self::open(&config.path)
    }
}
