// "Variable length quantity" derived from https://github.com/Klipper3d/klipper/blob/c707dd19214709dc23684b254a68e3bf69e4cfb3/src/command.c

use std::io::{Read, Write};

fn read_byte(reader: &mut dyn Read) -> anyhow::Result<u8> {
    let mut buf = [0u8; 1];
    reader.read_exact(&mut buf)?;
    Ok(buf[0])
}

/// Encode a 32-bit unsigned integer as a variable length quantity (VLQ) into a writer.
pub fn encode_int_to(v: u32, writer: &mut dyn Write) -> anyhow::Result<()> {
    let sv = v as i32;

    if sv < (3 << 5) && sv >= -(1 << 5) {
        writer.write_all(&[(v & 0x7f) as u8])?;
        return Ok(());
    }

    if sv < (3 << 12) && sv >= -(1 << 12) {
        writer.write_all(&[
            ((v >> 7) & 0x7f) as u8 | 0x80,
            (v & 0x7f) as u8,
        ])?;
        return Ok(());
    }

    if sv < (3 << 19) && sv >= -(1 << 19) {
        writer.write_all(&[
            ((v >> 14) & 0x7f) as u8 | 0x80,
            ((v >> 7) & 0x7f) as u8 | 0x80,
            (v & 0x7f) as u8,
        ])?;
        return Ok(());
    }

    if sv < (3 << 26) && sv >= -(1 << 26) {
        writer.write_all(&[
            ((v >> 21) & 0x7f) as u8 | 0x80,
            ((v >> 14) & 0x7f) as u8 | 0x80,
            ((v >> 7) & 0x7f) as u8 | 0x80,
            (v & 0x7f) as u8,
        ])?;
        return Ok(());
    }

    writer.write_all(&[
        ((v >> 28) & 0x7f) as u8 | 0x80,
        ((v >> 21) & 0x7f) as u8 | 0x80,
        ((v >> 14) & 0x7f) as u8 | 0x80,
        ((v >> 7) & 0x7f) as u8 | 0x80,
        (v & 0x7f) as u8,
    ])?;
    Ok(())
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

/// Encode a message ID (up to 16 bits) into a variable-length format in a writer.
pub fn encode_msgid_to(encoded_msgid: i16, writer: &mut dyn Write) -> anyhow::Result<()> {
    let v = encoded_msgid as u16;
    if v >= 0x80 {
        writer.write_all(&[((v >> 7) & 0x7f) as u8 | 0x80])?;
    }
    writer.write_all(&[(v & 0x7f) as u8])?;
    Ok(())
}

/// Encode a message ID (up to 16 bits) into a variable-length format.
/// Returns a Vec<u8> containing the encoded bytes (1 or 2 bytes).
pub fn encode_msgid(encoded_msgid: i16) -> Vec<u8> {
    let v = encoded_msgid as u16;
    let mut buf = Vec::with_capacity(2);

    if v >= 0x80 {
        buf.push(((v >> 7) & 0x7f) as u8 | 0x80);
    }
    buf.push((v & 0x7f) as u8);

    buf
}

/// Decode a variable-length encoded message ID from a reader.
pub fn parse_msgid(reader: &mut dyn Read) -> anyhow::Result<i16> {
    let first = read_byte(reader)?;
    let mut msgid = first as u16;

    if first & 0x80 != 0 {
        let second = read_byte(reader)?;
        msgid = ((first as u16 & 0x7f) << 7) | second as u16;
    }

    Ok(msgid as i16)
}
