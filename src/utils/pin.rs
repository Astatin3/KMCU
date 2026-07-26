use core::fmt;

#[derive(Eq)]
pub struct Pin {
    // Numerical representation of a pin
    pub num: u16,

    // String representation of a pin,
    // like PA16 or PB0
    // Should never be longer than 4 chars
    pub tag: heapless::String<4>,
}

fn pin_name_to_int(pin_name: &str) -> Option<u32> {
    let first_byte = pin_name.as_bytes()[1];

    if b'A' > first_byte {
        return None;
    }

    let port = first_byte - b'A';

    let pin_idx: u32 = pin_name[2..].parse().ok()?;

    Some(((port as u32) << 5) + pin_idx)
}

fn int_to_pin_name(num: u32) -> Option<heapless::String<4>> {
    let port = (num >> 5) as u8;
    let pin_idx = num & 0x1F;

    if port > 25 {
        return None;
    }

    let letter = (b'A' + port) as char;
    let mut s = heapless::String::<4>::new();

    // These pushes can never fail
    let _ = s.push('P');
    let _ = s.push(letter);

    // If we need another digit
    if pin_idx >= 10 {
        let _ = s.push((b'0' + (pin_idx / 10) as u8) as char);
    }

    let _ = s.push((b'0' + (pin_idx % 10) as u8) as char);
    Some(s)
}

impl Pin {
    pub fn from_str(pin_name: &str) -> Option<Self> {
        let num = pin_name_to_int(pin_name)? as u16;
        let mut tag = heapless::String::<4>::new();
        tag.push_str(pin_name).ok()?;
        Some(Self { num, tag })
    }

    pub fn from_num(num: u16) -> Option<Self> {
        let tag = int_to_pin_name(num as u32)?;
        Some(Self { num, tag })
    }
}

impl PartialEq for Pin {
    fn eq(&self, other: &Self) -> bool {
        self.num == other.num
    }
}

impl fmt::Display for Pin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.tag);
        Ok(())
    }
}
