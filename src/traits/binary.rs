use std::io::Read;

use bytes::BytesMut;

pub trait Binary: Sized {
    type EncodeArg;
    type DecodeArg;

    fn encode(&self, buf: &mut BytesMut, arg: Self::EncodeArg);
    fn decode(reader: &mut dyn Read, arg: Self::DecodeArg) -> anyhow::Result<Self>;
}

macro_rules! binary_vlq_unsigned {
    ($t:tt) => {
        impl Binary for $t {
            type EncodeArg = ();
            type DecodeArg = ();

            fn encode(&self, buf: &mut BytesMut, _: ()) {
                crate::wire::vlq::encode_int_to(*self as u32, buf);
            }

            fn decode(reader: &mut dyn Read, _: ()) -> anyhow::Result<Self> {
                let v = crate::wire::vlq::parse_int(reader)?;
                Ok(v as $t)
            }
        }
    };
}

macro_rules! binary_vlq_signed {
    ($t:tt) => {
        impl Binary for $t {
            type EncodeArg = ();
            type DecodeArg = ();

            fn encode(&self, buf: &mut BytesMut, _: ()) {
                crate::wire::vlq::encode_int_to(*self as u32, buf);
            }

            fn decode(reader: &mut dyn Read, _: ()) -> anyhow::Result<Self> {
                let v = crate::wire::vlq::parse_int(reader)?;
                Ok(v as $t)
            }
        }
    };
}

binary_vlq_unsigned!(u32);
binary_vlq_unsigned!(u16);
binary_vlq_unsigned!(u8);
binary_vlq_signed!(i32);
binary_vlq_signed!(i16);

#[allow(invalid_type_param_default)]
impl<T: Binary<EncodeArg = ()>> Binary for Vec<T> {
    type EncodeArg = ();
    type DecodeArg = ();

    fn encode(&self, buf: &mut BytesMut, _: ()) {
        for item in self {
            item.encode(buf, ());
        }
    }

    fn decode(_: &mut dyn Read, _: ()) -> anyhow::Result<Self> {
        unreachable!()
    }
}
