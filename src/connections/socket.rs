use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};

use crate::config::SocketConnection;
use crate::traits::from_config::FromConfig;

pub struct Socket {
    inner: File,
    config: SocketConnection,
}

impl Socket {
    pub fn new(config: SocketConnection) -> anyhow::Result<Self> {
        let inner = File::options().read(true).write(true).open(&config.path)?;
        debug!("Opened socket device '{}'", config.path);
        Ok(Self { inner, config })
    }

    pub fn new_serial(config: SocketConnection) -> anyhow::Result<Self> {
        let baud = config.baud.unwrap_or(115_200);

        let inner = unsafe {
            let raw_fd = libc::open(
                config.path.as_ptr() as *const libc::c_char,
                libc::O_RDWR | libc::O_NOCTTY,
            );
            if raw_fd < 0 {
                return Err(std::io::Error::last_os_error().into());
            }

            let mut termios: libc::termios = std::mem::zeroed();
            if libc::tcgetattr(raw_fd, &mut termios) != 0 {
                libc::close(raw_fd);
                return Err(std::io::Error::last_os_error().into());
            }

            libc::cfmakeraw(&mut termios);

            // Baud rate via BOTHER
            termios.c_cflag = (termios.c_cflag & !libc::CBAUD) | libc::BOTHER;
            libc::cfsetispeed(&mut termios, baud as libc::speed_t);
            libc::cfsetospeed(&mut termios, baud as libc::speed_t);

            // Non-blocking for poll-based timeout
            termios.c_cc[libc::VMIN] = 0;
            termios.c_cc[libc::VTIME] = 0;

            if libc::tcsetattr(raw_fd, libc::TCSANOW, &termios) != 0 {
                libc::close(raw_fd);
                return Err(std::io::Error::last_os_error().into());
            }

            // Set O_NONBLOCK for poll-based timeout
            let flags = libc::fcntl(raw_fd, libc::F_GETFL);
            libc::fcntl(raw_fd, libc::F_SETFL, flags | libc::O_NONBLOCK);

            File::from_raw_fd(raw_fd)
        };

        debug!(
            "Opened serial port '{}' at {baud} baud, timeout={:?}",
            config.path, config.timeout
        );
        Ok(Self { inner, config })
    }

    fn poll_ready(&self, fd: RawFd, events: i16) -> anyhow::Result<()> {
        let mut pollfd = libc::pollfd {
            fd,
            events,
            revents: 0,
        };

        let ms = self.config.timeout.as_millis() as i32;
        let ret = unsafe { libc::poll(&mut pollfd, 1, ms) };

        if ret < 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        if ret == 0 {
            anyhow::bail!("Timed out after {:?}", self.config.timeout);
        }
        if pollfd.revents & libc::POLLERR != 0 {
            anyhow::bail!("Poll error on socket device");
        }

        Ok(())
    }
}

impl Read for Socket {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.poll_ready(self.inner.as_raw_fd(), libc::POLLIN)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::TimedOut, e.to_string()))?;
        self.inner.read(buf)
    }
}

impl Write for Socket {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.poll_ready(self.inner.as_raw_fd(), libc::POLLOUT)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::TimedOut, e.to_string()))?;
        self.inner.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

impl crate::connections::Stream for Socket {}

impl FromConfig for Socket {
    type ConfigType = SocketConnection;

    fn from_config(config: Self::ConfigType) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        if config.baud.is_some() {
            Self::new_serial(config)
        } else {
            Self::new(config)
        }
    }
}
