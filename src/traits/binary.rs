use alloc::vec::{self, Vec};

use crate::{
    traits::{Read, Write},
    utils::error::IOError,
};

pub trait Binary: Sized {
    type EncodeArg;
    type DecodeArg;

    fn encode(&self, writer: &mut dyn Write, arg: &Self::EncodeArg) -> Result<(), IOError>;
    fn decode(reader: &mut dyn Read, arg: &Self::DecodeArg) -> Result<Self, IOError>;
    fn size(&self, arg: &Self::EncodeArg) -> usize;
}

macro_rules! binary_vlq_unsigned {
    ($t:tt) => {
        impl Binary for $t {
            type EncodeArg = ();
            type DecodeArg = ();

            fn encode(&self, writer: &mut dyn Write, _: &()) -> Result<(), IOError> {
                $crate::utils::vlq::encode_int_to(*self as u32, writer)
            }

            fn decode(reader: &mut dyn Read, _: &()) -> Result<Self, IOError> {
                let v = $crate::utils::vlq::parse_int(reader)?;
                Ok(v as $t)
            }

            fn size(&self, _: &()) -> usize {
                $crate::utils::vlq::vlq_int_size(*self as u32)
            }
        }
    };
}

macro_rules! binary_vlq_signed {
    ($t:tt) => {
        impl Binary for $t {
            type EncodeArg = ();
            type DecodeArg = ();

            fn encode(&self, writer: &mut dyn Write, _: &()) -> Result<(), IOError> {
                $crate::utils::vlq::encode_int_to(*self as u32, writer)
            }

            fn decode(reader: &mut dyn Read, _: &()) -> Result<Self, IOError> {
                let v = $crate::utils::vlq::parse_int(reader)?;
                Ok(v as $t)
            }

            fn size(&self, _: &()) -> usize {
                $crate::utils::vlq::vlq_int_size(*self as u32)
            }
        }
    };
}

binary_vlq_unsigned!(u32);
binary_vlq_unsigned!(u16);
binary_vlq_signed!(i32);
binary_vlq_signed!(i16);

impl Binary for u8 {
    type EncodeArg = ();
    type DecodeArg = ();

    fn encode(&self, writer: &mut dyn Write, _: &()) -> Result<(), IOError> {
        writer.write_all(&[*self])?;
        Ok(())
    }

    fn decode(reader: &mut dyn Read, _: &()) -> Result<Self, IOError> {
        let mut buf = [0u8; 1];
        reader.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    fn size(&self, _: &()) -> usize {
        1
    }
}

impl Binary for Vec<u8> {
    type EncodeArg = ();
    type DecodeArg = ();

    fn encode(&self, writer: &mut dyn Write, _: &()) -> Result<(), IOError> {
        writer.write_all(&[self.len() as u8])?;
        writer.write_all(self)?;
        Ok(())
    }

    fn decode(reader: &mut dyn Read, _: &()) -> Result<Self, IOError> {
        let mut len = [0u8; 1];
        reader.read_exact(&mut len)?;

        let len = len[0] as usize;
        let mut buf = Vec::with_capacity(len);
        buf.resize(len, 0);

        reader.read_exact(&mut buf)?;
        Ok(buf)
    }

    fn size(&self, _: &()) -> usize {
        self.len() + 1
    }
}
