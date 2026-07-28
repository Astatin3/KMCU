pub const CRC16_INITIAL: u16 = 0xffff;

pub fn crc16_update(mut crc: u16, byte: u8) -> u16 {
    let mut data: u16 = byte as u16;
    data ^= crc & 0xff;
    data ^= (data & 0x0f) << 4;
    crc = ((data << 8) | (crc >> 8)) ^ (data >> 4) ^ (data << 3);
    crc
}

pub fn crc16_final(crc: u16) -> [u8; 2] {
    [(crc >> 8) as u8, (crc & 0xff) as u8]
}

pub fn crc16_ccitt(buf: &[u8]) -> [u8; 2] {
    let mut crc = CRC16_INITIAL;
    for &byte in buf {
        crc = crc16_update(crc, byte);
    }
    crc16_final(crc)
}
