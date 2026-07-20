pub trait Binary: Sized {
    type DecodeArg;

    fn encode(&self) -> Vec<u8>;
    fn decode(buf: &[u8], arg: Self::DecodeArg) -> anyhow::Result<Self>;
}

fn slice_to_array<const N: usize>(slice: &[u8]) -> Result<[u8; N], std::array::TryFromSliceError> {
    // Performs a runtime size check and copies the data
    slice.try_into()
}

macro_rules! binary_num {
    ($t:tt) => {
        impl Binary for $t {
            type DecodeArg = ();

            fn encode(&self) -> Vec<u8> {
                self.to_le_bytes().to_vec()
            }

            fn decode(buf: &[u8], _: ()) -> anyhow::Result<Self> {
                Ok($t::from_le_bytes(slice_to_array(buf)?))
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

    fn decode(_: &[u8], _: ()) -> anyhow::Result<Self> {
        unreachable!()
    }
}
