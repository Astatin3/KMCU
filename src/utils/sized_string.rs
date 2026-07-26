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
        &self.0[N]
    }
}

impl<const N: usize> IndexMut<usize> for SizedString<N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[N]
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
