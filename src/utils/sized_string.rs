use core::{
    fmt::{Debug, Display, Write},
    ops::{Index, IndexMut},
};
use std::string::String;

pub struct SizedString<const N: usize>([char; N]);

impl<const N: usize> SizedString<N> {
    /// Create new array
    pub fn new() -> Self {
        Self(['\0'; N])
    }

    /// Get one char or return None
    pub fn get(&self, index: usize) -> Option<&char> {
        if index >= N { None } else { Some(&self[index]) }
    }

    /// Parses self as a i32
    pub fn parse_i32(&self) -> Option<i32> {
        let mut n = 0;

        // max is number of digits in the signed 32 bit integer limit plus a '-'
        let max = N.max(11);

        let (invert, range) = if self.0[0] == '-' {
            (true, 1..max)
        } else {
            (false, 0..max)
        };

        for i in range {
            n *= 10;

            n += match self.0[i] {
                '0' => 0,
                '1' => 1,
                '2' => 2,
                '3' => 3,
                '4' => 4,
                '5' => 5,
                '6' => 6,
                '7' => 7,
                '8' => 8,
                '9' => 9,
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

        self[index] = '\0'
    }
}

impl<const N: usize> From<[char; N]> for SizedString<N> {
    fn from(value: [char; N]) -> Self {
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

impl<const N: usize> From<&str> for SizedString<N> {
    fn from(value: &str) -> Self {
        let mut chars = value.chars();
        let mut array = ['\0'; N];

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
    type Output = char;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<const N: usize> IndexMut<usize> for SizedString<N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl<const N: usize> Display for SizedString<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for char in self.0 {
            // If it's not a null char
            if char != '\0' {
                f.write_char(char)?;
            }
        }
        Ok(())
    }
}

impl<const N: usize> Debug for SizedString<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for char in self.0 {
            // If it's not a null char
            if char != '\0' {
                f.write_char(char)?;
            }
        }
        Ok(())
    }
}

impl<const N: usize> FromIterator<char> for SizedString<N> {
    fn from_iter<I: IntoIterator<Item = char>>(iter: I) -> Self {
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
