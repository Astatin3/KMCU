use core::{
    fmt::{Debug, Display, Write},
    ops::{Index, IndexMut},
};
use std::string::String;

#[derive(Clone)]
pub struct SizedString<const N: usize>([u8; N]);

impl<const N: usize> SizedString<N> {
    /// Create new array
    pub fn new() -> Self {
        Self([0; N])
    }

    /// Get one char or return None
    pub fn get(&self, index: usize) -> Option<&u8> {
        if index >= N { None } else { Some(&self[index]) }
    }

    /// Parses self as a i32
    pub fn parse_i32(&self) -> Option<i32> {
        let mut n = 0;

        // max is number of digits in the signed 32 bit integer limit plus a '-'
        let max = N.max(11);

        let (invert, range) = if self.0[0] == b'-' {
            (true, 1..max)
        } else {
            (false, 0..max)
        };

        for i in range {
            n *= 10;

            n += match self.0[i] {
                b'0' => 0,
                b'1' => 1,
                b'2' => 2,
                b'3' => 3,
                b'4' => 4,
                b'5' => 5,
                b'6' => 6,
                b'7' => 7,
                b'8' => 8,
                b'9' => 9,
                _ => return None,
            }
        }

        if invert {
            n = -n;
        }

        Some(n)
    }

    /// Sets one char to null
    pub fn clear(&mut self, index: usize) {
        if index >= N {
            return;
        }

        self[index] = 0;
    }
}

impl<const N: usize> From<[u8; N]> for SizedString<N> {
    fn from(value: [u8; N]) -> Self {
        Self(value)
    }
}

impl<const N: usize> From<String> for SizedString<N> {
    fn from(value: String) -> Self {
        value.as_str().into()
    }
}

impl<const N: usize> From<&String> for SizedString<N> {
    fn from(value: &String) -> Self {
        value.as_str().into()
    }
}

impl<const N: usize> From<&mut String> for SizedString<N> {
    fn from(value: &mut String) -> Self {
        value.as_str().into()
    }
}

impl<const N: usize> From<&str> for SizedString<N> {
    fn from(value: &str) -> Self {
        let mut chars = value.bytes();
        let mut array = [0u8; N];

        for i in 0..N {
            match chars.next() {
                Some(c) => array[i] = c,
                None => break,
            }
        }

        Self(array)
    }
}

impl<const N: usize> Index<usize> for SizedString<N> {
    type Output = u8;

    fn index(&self, index: usize) -> &u8 {
        &self.0[index]
    }
}

impl<const N: usize> IndexMut<usize> for SizedString<N> {
    fn index_mut(&mut self, index: usize) -> &mut u8 {
        &mut self.0[index]
    }
}

impl<const N: usize> Display for SizedString<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let len = self.0.iter().position(|&b| b == 0).unwrap_or(N);
        let s = unsafe { core::str::from_utf8_unchecked(&self.0[..len]) };
        f.write_str(s)
    }
}

impl<const N: usize> Debug for SizedString<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let len = self.0.iter().position(|&b| b == 0).unwrap_or(N);
        let s = unsafe { core::str::from_utf8_unchecked(&self.0[..len]) };
        f.write_str(s)
    }
}

impl<const N: usize> FromIterator<u8> for SizedString<N> {
    fn from_iter<I: IntoIterator<Item = u8>>(iter: I) -> Self {
        let mut this = Self::new();

        let mut iter = iter.into_iter();

        for i in 0..N {
            match iter.next() {
                Some(c) => this[i] = c,
                None => return this,
            }
        }

        this
    }
}
