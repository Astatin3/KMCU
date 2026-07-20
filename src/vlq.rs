// "Variable length quantity" derived from https://github.com/Klipper3d/klipper/blob/c707dd19214709dc23684b254a68e3bf69e4cfb3/src/command.c

use std::io::Read;

use bytes::BufMut;

fn read_byte(reader: &mut dyn Read) -> anyhow::Result<u8> {
    let mut buf = [0u8; 1];
    reader.read_exact(&mut buf)?;
    Ok(buf[0])
}

/// Encode a 32-bit unsigned integer as a variable length quantity (VLQ) into a buffer.
pub fn encode_int_to(v: u32, buf: &mut bytes::BytesMut) {
    let sv = v as i32;

    if sv < (3 << 5) && sv >= -(1 << 5) {
        buf.put_u8((v & 0x7f) as u8);
        return;
    }

    if sv < (3 << 12) && sv >= -(1 << 12) {
        buf.put_u8(((v >> 7) & 0x7f) as u8 | 0x80);
        buf.put_u8((v & 0x7f) as u8);
        return;
    }

    if sv < (3 << 19) && sv >= -(1 << 19) {
        buf.put_u8(((v >> 14) & 0x7f) as u8 | 0x80);
        buf.put_u8(((v >> 7) & 0x7f) as u8 | 0x80);
        buf.put_u8((v & 0x7f) as u8);
        return;
    }

    if sv < (3 << 26) && sv >= -(1 << 26) {
        buf.put_u8(((v >> 21) & 0x7f) as u8 | 0x80);
        buf.put_u8(((v >> 14) & 0x7f) as u8 | 0x80);
        buf.put_u8(((v >> 7) & 0x7f) as u8 | 0x80);
        buf.put_u8((v & 0x7f) as u8);
        return;
    }

    buf.put_u8(((v >> 28) & 0x7f) as u8 | 0x80);
    buf.put_u8(((v >> 21) & 0x7f) as u8 | 0x80);
    buf.put_u8(((v >> 14) & 0x7f) as u8 | 0x80);
    buf.put_u8(((v >> 7) & 0x7f) as u8 | 0x80);
    buf.put_u8((v & 0x7f) as u8);
}

/// Encode a 32-bit unsigned integer as a variable length quantity (VLQ).
/// Returns a Vec<u8> containing the encoded bytes.
pub fn encode_int(v: u32) -> Vec<u8> {
    let sv = v as i32;
    let mut buf = Vec::with_capacity(5);

    if sv < (3 << 5) && sv >= -(1 << 5) {
        buf.push((v & 0x7f) as u8);
        return buf;
    }

    if sv < (3 << 12) && sv >= -(1 << 12) {
        buf.push(((v >> 7) & 0x7f) as u8 | 0x80);
        buf.push((v & 0x7f) as u8);
        return buf;
    }

    if sv < (3 << 19) && sv >= -(1 << 19) {
        buf.push(((v >> 14) & 0x7f) as u8 | 0x80);
        buf.push(((v >> 7) & 0x7f) as u8 | 0x80);
        buf.push((v & 0x7f) as u8);
        return buf;
    }

    if sv < (3 << 26) && sv >= -(1 << 26) {
        buf.push(((v >> 21) & 0x7f) as u8 | 0x80);
        buf.push(((v >> 14) & 0x7f) as u8 | 0x80);
        buf.push(((v >> 7) & 0x7f) as u8 | 0x80);
        buf.push((v & 0x7f) as u8);
        return buf;
    }

    buf.push(((v >> 28) & 0x7f) as u8 | 0x80);
    buf.push(((v >> 21) & 0x7f) as u8 | 0x80);
    buf.push(((v >> 14) & 0x7f) as u8 | 0x80);
    buf.push(((v >> 7) & 0x7f) as u8 | 0x80);
    buf.push((v & 0x7f) as u8);
    buf
}

/// Decode a VLQ-encoded integer from a reader.
pub fn parse_int(reader: &mut dyn Read) -> anyhow::Result<u32> {
    let mut c = read_byte(reader)?;
    let mut v = (c & 0x7f) as u32;

    if (c & 0x60) == 0x60 {
        v |= !0x1F;
    }

    while c & 0x80 != 0 {
        c = read_byte(reader)?;
        v = (v << 7) | (c & 0x7f) as u32;
    }

    Ok(v)
}

/// Encode a message ID (up to 16 bits) into a variable-length format in a buffer.
pub fn encode_msgid_to(encoded_msgid: u16, buf: &mut bytes::BytesMut) {
    if encoded_msgid >= 0x80 {
        buf.put_u8(((encoded_msgid >> 7) & 0x7f) as u8 | 0x80);
    }
    buf.put_u8((encoded_msgid & 0x7f) as u8);
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

/// Decode a variable-length encoded message ID from a reader.
pub fn parse_msgid(reader: &mut dyn Read) -> anyhow::Result<u16> {
    let first = read_byte(reader)?;
    let mut msgid = first as u16;

    if first & 0x80 != 0 {
        let second = read_byte(reader)?;
        msgid = ((first as u16 & 0x7f) << 7) | second as u16;
    }

    Ok(msgid)
}
