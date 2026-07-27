use std::fs::File;
use std::io::{Read as _, Write as _};
use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};

use crate::config::{SerialConnection, SocketConnection};
use crate::traits::{Read, Stream, Write};
use crate::utils::error::IOError;
use crate::utils::units;

pub struct Socket {
    fd: File,
    config: SocketConnection,
}

impl Socket {
    pub fn new(config: SocketConnection) -> Result<Self, IOError> {
        let fd = File::options().read(true).write(true).open(&config.path)?;
        debug!("Opened socket device '{}'", config.path);
        Ok(Self { fd, config })
    }

    pub fn new_serial(config: SerialConnection) -> Result<Self, IOError> {
        let fd = unsafe {
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
            let baud = config.baud as libc::speed_t;
            termios.c_cflag = (termios.c_cflag & !libc::CBAUD) | libc::BOTHER;
            libc::cfsetispeed(&mut termios, baud);
            libc::cfsetospeed(&mut termios, baud);

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
            "Opened serial port '{}' at {} baud, timeout={:?}",
            config.baud, config.path, config.timeout
        );

        Ok(Self {
            fd,
            config: SocketConnection {
                path: config.path,
                timeout: config.timeout,
            },
        })
    }

    fn poll_ready(&self, fd: RawFd, events: i16) -> Result<(), IOError> {
        let mut pollfd = libc::pollfd {
            fd,
            events,
            revents: 0,
        };

        let ms = self.config.timeout.get::<units::long_millisecond>() as i32;
        let ret = unsafe { libc::poll(&mut pollfd, 1, ms) };

        if ret < 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        if ret == 0 {
            return Err(IOError::Timeout);
        }
        if pollfd.revents & libc::POLLERR != 0 {
            return Err(IOError::Linux { errno: 0 });
        }

        Ok(())
    }
}

impl Read for Socket {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, IOError> {
        self.poll_ready(self.fd.as_raw_fd(), libc::POLLIN)?;
        Ok(std::io::Read::read(&mut self.fd, buf).map_err(IOError::from)?)
    }

    fn flush_input(&mut self) -> Result<(), IOError> {
        let fd = self.fd.as_raw_fd();
        unsafe { libc::tcflush(fd, libc::TCIFLUSH) };
        Ok(())
    }
}

impl Write for Socket {
    fn write(&mut self, buf: &[u8]) -> Result<usize, IOError> {
        self.poll_ready(self.fd.as_raw_fd(), libc::POLLOUT)?;
        Ok(std::io::Write::write(&mut self.fd, buf).map_err(IOError::from)?)
    }

    fn flush_output(&mut self) -> Result<(), IOError> {
        std::io::Write::flush(&mut self.fd).map_err(IOError::from)?;
        Ok(())
    }
}

impl Stream for Socket {}
