use alloc::vec::{self, Vec};

use crate::{
    error::Res,
    traits::{Read, Write},
};

pub trait Binary: Sized {
    type EncodeArg;
    type DecodeArg;

    fn encode(&self, writer: &mut dyn Write, arg: &Self::EncodeArg) -> Res<()>;
    fn decode(reader: &mut dyn Read, arg: &Self::DecodeArg) -> Res<Self>;
    fn size(&self, arg: &Self::EncodeArg) -> usize;
}

macro_rules! binary_vlq_unsigned {
    ($t:tt) => {
        impl Binary for $t {
            type EncodeArg = ();
            type DecodeArg = ();

            fn encode(&self, writer: &mut dyn Write, _: &()) -> Res<()> {
                $crate::vlq::encode_int_to(*self as u32, writer)
            }

            fn decode(reader: &mut dyn Read, _: &()) -> Res<Self> {
                let v = $crate::vlq::parse_int(reader)?;
                Ok(v as $t)
            }

            fn size(&self, _: &()) -> usize {
                $crate::vlq::vlq_int_size(*self as u32)
            }
        }
    };
}

macro_rules! binary_vlq_signed {
    ($t:tt) => {
        impl Binary for $t {
            type EncodeArg = ();
            type DecodeArg = ();

            fn encode(&self, writer: &mut dyn Write, _: &()) -> Res<()> {
                $crate::vlq::encode_int_to(*self as u32, writer)
            }

            fn decode(reader: &mut dyn Read, _: &()) -> Res<Self> {
                let v = $crate::vlq::parse_int(reader)?;
                Ok(v as $t)
            }

            fn size(&self, _: &()) -> usize {
                $crate::vlq::vlq_int_size(*self as u32)
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

    fn encode(&self, writer: &mut dyn Write, _: &()) -> Res<()> {
        writer.write_all(&[*self])?;
        Ok(())
    }

    fn decode(reader: &mut dyn Read, _: &()) -> Res<Self> {
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

    fn encode(&self, writer: &mut dyn Write, _: &()) -> Res<()> {
        writer.write_all(&[self.len() as u8])?;
        writer.write_all(self)?;
        Ok(())
    }

    fn decode(reader: &mut dyn Read, _: &()) -> Res<Self> {
        let mut len = [0u8; 1];
        reader.read_exact(&mut len)?;
        let mut buf = vec![0u8; len[0] as usize];
        reader.read_exact(&mut buf)?;
        Ok(buf)
    }

    fn size(&self, _: &()) -> usize {
        self.len() + 1
    }
}
