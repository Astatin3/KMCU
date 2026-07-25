use std::fs::File;
use std::io::{Read as _, Write as _};
use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};

use crate::units;

use crate::config::{SerialConnection, SocketConnection};
use crate::error::Res;
use crate::traits::{Read, Stream, Write};

pub struct Socket {
    fd: File,
    config: SocketConnection,
}

impl Socket {
    pub fn new(config: SocketConnection) -> Res<Self> {
        let fd = File::options()
            .read(true)
            .write(true)
            .open(&config.path)
            .map_err(|e| err!("{e}"))?;
        debug!("Opened socket device '{}'", config.path);
        Ok(Self { fd, config })
    }

    pub fn new_serial(config: SerialConnection) -> Res<Self> {
        let fd = unsafe {
            let raw_fd = libc::open(
                config.path.as_ptr() as *const libc::c_char,
                libc::O_RDWR | libc::O_NOCTTY,
            );
            if raw_fd < 0 {
                return Err(err!("{}", std::io::Error::last_os_error()));
            }

            let mut termios: libc::termios = std::mem::zeroed();
            if libc::tcgetattr(raw_fd, &mut termios) != 0 {
                libc::close(raw_fd);
                return Err(err!("{}", std::io::Error::last_os_error()));
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
                return Err(err!("{}", std::io::Error::last_os_error()));
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

    fn poll_ready(&self, fd: RawFd, events: i16) -> Res<()> {
        let mut pollfd = libc::pollfd {
            fd,
            events,
            revents: 0,
        };

        let ms = self.config.timeout.get::<units::long_millisecond>() as i32;
        let ret = unsafe { libc::poll(&mut pollfd, 1, ms) };

        if ret < 0 {
            return Err(err!("{}", std::io::Error::last_os_error()));
        }
        if ret == 0 {
            return Err(err!("Timed out after {:?}", self.config.timeout));
        }
        if pollfd.revents & libc::POLLERR != 0 {
            return Err(err!("Poll error on socket device"));
        }

        Ok(())
    }
}

impl Read for Socket {
    fn read(&mut self, buf: &mut [u8]) -> Res<usize> {
        self.poll_ready(self.fd.as_raw_fd(), libc::POLLIN)?;
        Ok(std::io::Read::read(&mut self.fd, buf).map_err(|e| err!("{e}"))?)
    }
}

impl Write for Socket {
    fn write(&mut self, buf: &[u8]) -> Res<usize> {
        self.poll_ready(self.fd.as_raw_fd(), libc::POLLOUT)?;
        Ok(self.fd.write(buf).map_err(|e| err!("{e}"))?)
    }

    fn flush(&mut self) -> Res<()> {
        Ok(self.fd.flush().map_err(|e| err!("{e}"))?)
    }
}

impl Stream for Socket {}
