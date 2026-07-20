use std::io::Read;

pub trait Binary: Sized {
    type DecodeArg;

    fn encode(&self) -> Vec<u8>;
    fn decode(reader: &mut dyn Read, arg: Self::DecodeArg) -> anyhow::Result<Self>;
}

macro_rules! binary_num {
    ($t:tt) => {
        impl Binary for $t {
            type DecodeArg = ();

            fn encode(&self) -> Vec<u8> {
                self.to_le_bytes().to_vec()
            }

            fn decode(reader: &mut dyn Read, _: ()) -> anyhow::Result<Self> {
                let mut buf = [0u8; std::mem::size_of::<$t>()];
                reader.read_exact(&mut buf)?;
                Ok($t::from_le_bytes(buf))
            }
        }
    };
}

// Define binary trait for types
binary_num!(u32);
binary_num!(i32);
binary_num!(u16);
binary_num!(i16);
binary_num!(u8);

// Useful for collections of binary formats
impl<T: Binary> Binary for Vec<T> {
    type DecodeArg = ();

    fn encode(&self) -> Vec<u8> {
        self.into_iter().map(|a| a.encode()).flatten().collect()
    }

    fn decode(_: &mut dyn Read, _: ()) -> anyhow::Result<Self> {
        unreachable!()
    }
}
