use core::fmt;

use serde::de::{self, Deserialize, Deserializer, Visitor};

use crate::utils::SizedString;

#[derive(Clone, Debug)]
pub struct Pin<const N: usize> {
    pub num: u8,
    pub tag: SizedString<4>,
    pub invert: u8,
}

fn strip_prefixes(s: &str) -> (u8, &str) {
    let bytes = s.as_bytes();
    let mut invert = 0u8;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'^' | b'~' => i += 1,
            b'!' if invert < 2 => {
                invert += 1;
                i += 1;
            }
            _ => break,
        }
    }
    (invert, &s[i..])
}

fn pin_name_to_int<const N: usize>(pin_name: &str) -> Option<u8> {
    let bytes = pin_name.as_bytes();
    if bytes.len() < 3 || bytes[0] != b'P' {
        return None;
    }
    let port_letter = bytes[1];
    if !(b'A'..=b'Z').contains(&port_letter) {
        return None;
    }
    let port = port_letter - b'A';
    let pin_idx: u8 = pin_name[2..].parse().ok()?;
    Some((port << N) + pin_idx)
}

fn int_to_pin_name<const N: usize>(num: u32) -> Option<SizedString<4>> {
    let port = (num >> N) as u8;
    let mask = (1u8 << N) - 1;
    let pin_idx = (num as u8) & mask;

    if port > 25 {
        return None;
    }

    let letter = (b'A' + port) as char;
    let mut buf = [0u8; 4];
    buf[0] = b'P';
    buf[1] = letter as u8;
    if pin_idx >= 10 {
        buf[2] = b'0' + (pin_idx / 10);
    }
    buf[3] = b'0' + (pin_idx % 10);
    Some(SizedString::from(buf))
}

impl<const N: usize> Pin<N> {
    pub fn from_str(pin_name: &str) -> Option<Self> {
        let (invert, rest) = strip_prefixes(pin_name);
        let num = pin_name_to_int::<N>(rest)?;
        let tag = SizedString::<4>::from(rest);
        Some(Self { num, tag, invert })
    }

    pub fn from_num(num: u8) -> Option<Self> {
        let tag = int_to_pin_name::<N>(num as u32)?;
        Some(Self {
            num,
            tag,
            invert: 0,
        })
    }
}

impl<const N: usize> PartialEq for Pin<N> {
    fn eq(&self, other: &Self) -> bool {
        self.num == other.num
    }
}

impl<const N: usize> Eq for Pin<N> {}

impl<const N: usize> fmt::Display for Pin<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.tag)
    }
}

impl<'de, const N: usize> Deserialize<'de> for Pin<N> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct PinVisitor<const M: usize>;

        impl<'de, const M: usize> Visitor<'de> for PinVisitor<M> {
            type Value = Pin<M>;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a pin string like \"PG16\", \"!PG17\", or \"^PB10\"")
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Pin<M>, E> {
                let (invert, rest) = strip_prefixes(v);
                let num =
                    pin_name_to_int::<M>(rest).ok_or_else(|| E::custom("invalid pin name"))?;
                let tag = SizedString::<4>::from(rest);
                Ok(Pin { num, tag, invert })
            }
        }

        deserializer.deserialize_str(PinVisitor::<N>)
    }
}
