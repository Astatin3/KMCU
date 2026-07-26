// "Variable length quantity" derived from https://github.com/Klipper3d/klipper/blob/c707dd19214709dc23684b254a68e3bf69e4cfb3/src/command.c

use crate::{
    traits::{Read, Write},
    utils::error::IOError,
};

fn read_byte(reader: &mut dyn Read) -> Result<u8, IOError> {
    let mut buf = [0u8; 1];
    reader.read_exact(&mut buf)?;
    Ok(buf[0])
}

/// Encode a 32-bit unsigned integer as a variable length quantity (VLQ) into a writer.
pub fn encode_int_to(v: u32, writer: &mut dyn Write) -> Result<(), IOError> {
    let sv = v as i32;

    if sv < (3 << 5) && sv >= -(1 << 5) {
        writer.write_all(&[(v & 0x7f) as u8])?;
        return Ok(());
    }

    if sv < (3 << 12) && sv >= -(1 << 12) {
        writer.write_all(&[((v >> 7) & 0x7f) as u8 | 0x80, (v & 0x7f) as u8])?;
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

/// Return the number of bytes `encode_int_to` would write for the given value.
pub fn vlq_int_size(v: u32) -> usize {
    let sv = v as i32;
    if sv < (3 << 5) && sv >= -(1 << 5) {
        1
    } else if sv < (3 << 12) && sv >= -(1 << 12) {
        2
    } else if sv < (3 << 19) && sv >= -(1 << 19) {
        3
    } else if sv < (3 << 26) && sv >= -(1 << 26) {
        4
    } else {
        5
    }
}

/// Decode a VLQ-encoded integer from a reader.
pub fn parse_int(reader: &mut dyn Read) -> Result<u32, IOError> {
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
pub fn encode_msgid_to(encoded_msgid: i16, writer: &mut dyn Write) -> Result<(), IOError> {
    let v = encoded_msgid as u16;
    if v >= 0x80 {
        writer.write_all(&[((v >> 7) & 0x7f) as u8 | 0x80])?;
    }
    writer.write_all(&[(v & 0x7f) as u8])?;
    Ok(())
}

/// Decode a variable-length encoded message ID from a reader.
pub fn parse_msgid(reader: &mut dyn Read) -> Result<i16, IOError> {
    let first = read_byte(reader)?;
    let mut msgid = first as u16;

    if first & 0x80 != 0 {
        let second = read_byte(reader)?;
        msgid = ((first as u16 & 0x7f) << 7) | second as u16;
    }

    Ok(msgid as i16)
}
