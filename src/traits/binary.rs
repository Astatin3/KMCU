use crate::traits::{Read, Write};

pub trait Binary: Sized {
    type EncodeArg;
    type DecodeArg;

    fn encode(&self, writer: &mut dyn Write, arg: &Self::EncodeArg) -> anyhow::Result<()>;
    fn decode(reader: &mut dyn Read, arg: &Self::DecodeArg) -> anyhow::Result<Self>;
    fn size(&self, arg: &Self::EncodeArg) -> usize;
}

macro_rules! binary_vlq_unsigned {
    ($t:tt) => {
        impl Binary for $t {
            type EncodeArg = ();
            type DecodeArg = ();

            fn encode(&self, writer: &mut dyn Write, _: &()) -> anyhow::Result<()> {
                crate::runtime::klipper_mcu::protocol::vlq::encode_int_to(*self as u32, writer)
            }

            fn decode(reader: &mut dyn Read, _: &()) -> anyhow::Result<Self> {
                let v = crate::runtime::klipper_mcu::protocol::vlq::parse_int(reader)?;
                Ok(v as $t)
            }

            fn size(&self, _: &()) -> usize {
                crate::runtime::klipper_mcu::protocol::vlq::vlq_int_size(*self as u32)
            }
        }
    };
}

macro_rules! binary_vlq_signed {
    ($t:tt) => {
        impl Binary for $t {
            type EncodeArg = ();
            type DecodeArg = ();

            fn encode(&self, writer: &mut dyn Write, _: &()) -> anyhow::Result<()> {
                crate::runtime::klipper_mcu::protocol::vlq::encode_int_to(*self as u32, writer)
            }

            fn decode(reader: &mut dyn Read, _: &()) -> anyhow::Result<Self> {
                let v = crate::runtime::klipper_mcu::protocol::vlq::parse_int(reader)?;
                Ok(v as $t)
            }

            fn size(&self, _: &()) -> usize {
                crate::runtime::klipper_mcu::protocol::vlq::vlq_int_size(*self as u32)
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

    fn encode(&self, writer: &mut dyn Write, _: &()) -> anyhow::Result<()> {
        writer.write_all(&[*self])?;
        Ok(())
    }

    fn decode(reader: &mut dyn Read, _: &()) -> anyhow::Result<Self> {
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

    fn encode(&self, writer: &mut dyn Write, _: &()) -> anyhow::Result<()> {
        writer.write_all(&[self.len() as u8])?;
        writer.write_all(self)?;
        Ok(())
    }

    fn decode(reader: &mut dyn Read, _: &()) -> anyhow::Result<Self> {
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
