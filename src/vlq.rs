// "Variable length quantity" derived from https://github.com/Klipper3d/klipper/blob/c707dd19214709dc23684b254a68e3bf69e4cfb3/src/command.c

/// Encode a 32-bit unsigned integer as a variable length quantity (VLQ).
/// Returns a Vec<u8> containing the encoded bytes.
pub fn encode_int(v: u32) -> Vec<u8> {
    let sv = v as i32;
    let mut buf = Vec::with_capacity(5);

    if sv < (3 << 5) && sv >= -(1 << 5) {
        // f4: 1 byte
        buf.push((v & 0x7f) as u8);
        return buf;
    }

    if sv < (3 << 12) && sv >= -(1 << 12) {
        // f3: 2 bytes
        buf.push(((v >> 7) & 0x7f) as u8 | 0x80);
        buf.push((v & 0x7f) as u8);
        return buf;
    }

    if sv < (3 << 19) && sv >= -(1 << 19) {
        // f2: 3 bytes
        buf.push(((v >> 14) & 0x7f) as u8 | 0x80);
        buf.push(((v >> 7) & 0x7f) as u8 | 0x80);
        buf.push((v & 0x7f) as u8);
        return buf;
    }

    if sv < (3 << 26) && sv >= -(1 << 26) {
        // f1: 4 bytes
        buf.push(((v >> 21) & 0x7f) as u8 | 0x80);
        buf.push(((v >> 14) & 0x7f) as u8 | 0x80);
        buf.push(((v >> 7) & 0x7f) as u8 | 0x80);
        buf.push((v & 0x7f) as u8);
        return buf;
    }

    // Full 5-byte encoding
    buf.push(((v >> 28) & 0x7f) as u8 | 0x80);
    buf.push(((v >> 21) & 0x7f) as u8 | 0x80);
    buf.push(((v >> 14) & 0x7f) as u8 | 0x80);
    buf.push(((v >> 7) & 0x7f) as u8 | 0x80);
    buf.push((v & 0x7f) as u8);
    buf
}

/// Parse a VLQ-encoded integer from a byte slice.
/// Returns the decoded value and the number of bytes consumed.
pub fn parse_int(data: &[u8]) -> Option<(u32, usize)> {
    let mut i = 0;
    let mut c = data.get(i)?;
    i += 1;

    let mut v = (c & 0x7f) as u32;

    if (c & 0x60) == 0x60 {
        v |= !0x1F; // Sign-extend for negative numbers
    }

    while c & 0x80 != 0 {
        c = data.get(i)?;
        i += 1;
        v = (v << 7) | (c & 0x7f) as u32;
    }

    Some((v, i))
}

/// Encode a message ID (up to 16 bits) into a variable-length format.
/// Returns a Vec<u8> containing the encoded bytes (1 or 2 bytes).
pub fn encode_msgid(encoded_msgid: u16) -> Vec<u8> {
    let mut buf = Vec::with_capacity(2);

    if encoded_msgid >= 0x80 {
        buf.push(((encoded_msgid >> 7) & 0x7f) as u8 | 0x80);
    }
    buf.push((encoded_msgid & 0x7f) as u8);

    buf
}

/// Parse a variable-length encoded message ID from a byte slice.
/// Returns the decoded message ID and the number of bytes consumed.
pub fn command_parse_msgid(data: &[u8]) -> Option<(u16, usize)> {
    let mut i = 0;
    let mut encoded_msgid = *data.get(i)? as u16;
    i += 1;

    if encoded_msgid & 0x80 != 0 {
        let low_byte = *data.get(i)? as u16;
        i += 1;
        encoded_msgid = ((encoded_msgid & 0x7f) << 7) | low_byte;
    }

    Some((encoded_msgid, i))
}
