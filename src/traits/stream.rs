use core::fmt;

use alloc::{string::String, vec::Vec};

use crate::error::Res;

pub trait Stream: Read + Write {}

pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Res<usize>;

    fn is_read_vectored(&self) -> bool {
        false
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Res<usize> {
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
                Err(e) => return Err(e),
            }
        }
    }

    fn read_to_string(&mut self, buf: &mut String) -> Res<usize> {
        let mut bytes = Vec::new();
        let len = self.read_to_end(&mut bytes)?;
        let s =
            core::str::from_utf8(&bytes).map_err(|_| err!("stream did not contain valid UTF-8"))?;
        buf.push_str(s);
        Ok(len)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Res<()> {
        let mut pos = 0;
        while pos < buf.len() {
            match self.read(&mut buf[pos..]) {
                Ok(0) => {
                    return Err(err!("failed to fill whole buffer"));
                }
                Ok(n) => pos += n,
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
    fn write(&mut self, buf: &[u8]) -> Res<usize>;
    fn flush(&mut self) -> Res<()>;

    fn is_write_vectored(&self) -> bool {
        false
    }

    fn write_all(&mut self, mut buf: &[u8]) -> Res<()> {
        while !buf.is_empty() {
            match self.write(buf) {
                Ok(0) => {
                    return Err(err!("failed to fill whole buffer"));
                }
                Ok(n) => buf = &buf[n..],
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) -> Res<()> {
        struct Adaptor<'a, W: ?Sized + 'a> {
            inner: &'a mut W,
            err: Res<()>,
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
        fmt::write(&mut adaptor, fmt).map_err(|e| err!("{e}"))?;
        adaptor.err
    }

    fn by_ref(&mut self) -> &mut Self
    where
        Self: Sized,
    {
        self
    }
}

impl Read for &[u8] {
    fn read(&mut self, buf: &mut [u8]) -> Res<usize> {
        let amt = buf.len().min(self.len());
        let (a, b) = self.split_at(amt);
        buf[..amt].copy_from_slice(a);
        *self = b;
        Ok(amt)
    }
}
