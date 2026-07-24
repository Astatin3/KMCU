use core::fmt;

pub trait Stream: Read + Write {}

pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> anyhow::Result<usize>;

    fn is_read_vectored(&self) -> bool {
        false
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> anyhow::Result<usize> {
        let start_len = buf.len();
        loop {
            if buf.len() == buf.capacity() {
                buf.reserve(32);
            }
            let spare = buf.spare_capacity_mut();
            let read_buf = unsafe {
                core::slice::from_raw_parts_mut(spare.as_mut_ptr().cast::<u8>(), spare.len())
            };
            match self.read(read_buf) {
                Ok(0) => return Ok(buf.len() - start_len),
                Ok(n) => unsafe {
                    buf.set_len(buf.len() + n);
                },
                Err(ref e) if is_interrupted(e) => continue,
                Err(e) => return Err(e),
            }
        }
    }

    fn read_to_string(&mut self, buf: &mut String) -> anyhow::Result<usize> {
        let mut bytes = Vec::new();
        let len = self.read_to_end(&mut bytes)?;
        let s = core::str::from_utf8(&bytes).map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "stream did not contain valid UTF-8",
            )
        })?;
        buf.push_str(s);
        Ok(len)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> anyhow::Result<()> {
        let mut pos = 0;
        while pos < buf.len() {
            match self.read(&mut buf[pos..]) {
                Ok(0) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        "failed to fill whole buffer",
                    )
                    .into());
                }
                Ok(n) => pos += n,
                Err(ref e) if is_interrupted(e) => continue,
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    fn by_ref(&mut self) -> &mut Self
    where
        Self: Sized,
    {
        self
    }
}

pub trait Write {
    fn write(&mut self, buf: &[u8]) -> anyhow::Result<usize>;
    fn flush(&mut self) -> anyhow::Result<()>;

    fn is_write_vectored(&self) -> bool {
        false
    }

    fn write_all(&mut self, mut buf: &[u8]) -> anyhow::Result<()> {
        while !buf.is_empty() {
            match self.write(buf) {
                Ok(0) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::WriteZero,
                        "failed to write whole buffer",
                    )
                    .into());
                }
                Ok(n) => buf = &buf[n..],
                Err(ref e) if is_interrupted(e) => continue,
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) -> anyhow::Result<()> {
        struct Adaptor<'a, W: ?Sized + 'a> {
            inner: &'a mut W,
            err: anyhow::Result<()>,
        }

        impl<W: Write + ?Sized> fmt::Write for Adaptor<'_, W> {
            fn write_str(&mut self, s: &str) -> fmt::Result {
                match self.inner.write_all(s.as_bytes()) {
                    Ok(()) => Ok(()),
                    Err(e) => {
                        self.err = Err(e);
                        Err(fmt::Error)
                    }
                }
            }
        }

        let mut adaptor = Adaptor {
            inner: self,
            err: Ok(()),
        };
        fmt::write(&mut adaptor, fmt)?;
        adaptor.err
    }

    fn by_ref(&mut self) -> &mut Self
    where
        Self: Sized,
    {
        self
    }
}

fn is_interrupted(e: &anyhow::Error) -> bool {
    e.downcast_ref::<std::io::Error>()
        .map_or(false, |e| e.kind() == std::io::ErrorKind::Interrupted)
}

impl<T: std::io::Read> Read for T {
    fn read(&mut self, buf: &mut [u8]) -> anyhow::Result<usize> {
        Ok(std::io::Read::read(self, buf)?)
    }

    fn is_read_vectored(&self) -> bool {
        false
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> anyhow::Result<usize> {
        Ok(std::io::Read::read_to_end(self, buf)?)
    }

    fn read_to_string(&mut self, buf: &mut String) -> anyhow::Result<usize> {
        Ok(std::io::Read::read_to_string(self, buf)?)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> anyhow::Result<()> {
        Ok(std::io::Read::read_exact(self, buf)?)
    }
}

impl<T: std::io::Write> Write for T {
    fn write(&mut self, buf: &[u8]) -> anyhow::Result<usize> {
        Ok(std::io::Write::write(self, buf)?)
    }

    fn flush(&mut self) -> anyhow::Result<()> {
        Ok(std::io::Write::flush(self)?)
    }

    fn is_write_vectored(&self) -> bool {
        false
    }

    fn write_all(&mut self, buf: &[u8]) -> anyhow::Result<()> {
        Ok(std::io::Write::write_all(self, buf)?)
    }

    fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) -> anyhow::Result<()> {
        Ok(std::io::Write::write_fmt(self, fmt)?)
    }
}
