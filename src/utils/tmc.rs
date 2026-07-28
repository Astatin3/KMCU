use alloc::vec::Vec;

const TMC_SYNC: u8 = 0xF5;

pub fn tmc_crc8(data: &[u8]) -> u8 {
    let mut crc = 0u8;
    for &b in data {
        let mut byte = b;
        for _ in 0..8 {
            if (crc >> 7) ^ (byte & 1) != 0 {
                crc = (crc << 1) ^ 0x07;
            } else {
                crc <<= 1;
            }
            byte >>= 1;
        }
    }
    crc
}

pub fn add_serial_bits(data: &[u8]) -> Vec<u8> {
    let total_bits = data.len() * 10;
    let byte_len = (total_bits + 7) / 8;
    let mut result = Vec::with_capacity(byte_len);
    result.resize(byte_len, 0u8);

    for (i, &d) in data.iter().enumerate() {
        let p = i * 10;
        let frame: u16 = ((d as u16) << 1) | 0x200;
        for k in 0..10 {
            if (frame >> k) & 1 == 1 {
                let bit_pos = p + k;
                result[bit_pos / 8] |= 1 << (bit_pos % 8);
            }
        }
    }

    result
}

fn get_frame(data: &[u8], frame_idx: usize) -> u16 {
    let p = frame_idx * 10;
    let mut frame = 0u16;
    for k in 0..10 {
        let bit_pos = p + k;
        let byte_idx = bit_pos / 8;
        let bit = bit_pos % 8;
        if byte_idx < data.len() && (data[byte_idx] >> bit) & 1 == 1 {
            frame |= 1 << k;
        }
    }
    frame
}

pub fn remove_serial_bits(data: &[u8], count: usize) -> Vec<u8> {
    let mut result = Vec::with_capacity(count);
    for i in 0..count {
        let frame = get_frame(data, i);
        let d = ((frame >> 1) & 0xFF) as u8;
        result.push(d);
    }
    result
}

pub fn build_write_datagram(addr: u8, reg: u8, val: u32) -> [u8; 8] {
    let mut msg = [0u8; 8];
    msg[0] = TMC_SYNC;
    msg[1] = addr;
    msg[2] = reg | 0x80;
    msg[3] = (val >> 24) as u8;
    msg[4] = (val >> 16) as u8;
    msg[5] = (val >> 8) as u8;
    msg[6] = val as u8;
    msg[7] = tmc_crc8(&msg[..7]);
    msg
}

pub fn build_read_datagram(addr: u8, reg: u8) -> [u8; 4] {
    let mut msg = [0u8; 4];
    msg[0] = TMC_SYNC;
    msg[1] = addr;
    msg[2] = reg & 0x7F;
    msg[3] = tmc_crc8(&msg[..3]);
    msg
}

pub fn encode_write(addr: u8, reg: u8, val: u32) -> Vec<u8> {
    let raw = build_write_datagram(addr, reg, val);
    add_serial_bits(&raw)
}

pub fn calc_current_bits(run_current: f32, sense_resistor: f32) -> (bool, u8) {
    let vsense = true;
    let cs = calc_cs(run_current, sense_resistor, true);

    if cs == 31 {
        let cur_at_31 = current_from_cs(31, sense_resistor, true);
        if cur_at_31 < run_current {
            let cs_vsense0 = calc_cs(run_current, sense_resistor, false);
            let cur_at_cs2 = current_from_cs(cs_vsense0, sense_resistor, false);
            if (run_current - cur_at_cs2).abs() < (run_current - cur_at_31).abs() {
                return (false, cs_vsense0);
            }
        }
    }

    (vsense, cs)
}

fn calc_cs(current: f32, sense_resistor: f32, vsense: bool) -> u8 {
    let vref = if vsense { 0.18 } else { 0.32 };
    let rs = sense_resistor + 0.020;
    let cs = (32.0 * rs * current * core::f32::consts::SQRT_2 / vref + 0.5) as i32 - 1;
    cs.clamp(0, 31) as u8
}

fn current_from_cs(cs: u8, sense_resistor: f32, vsense: bool) -> f32 {
    let vref = if vsense { 0.18 } else { 0.32 };
    let rs = sense_resistor + 0.020;
    (cs as f32 + 1.0) * vref / (32.0 * rs * core::f32::consts::SQRT_2)
}

pub fn calc_hold_current_bits(hold_current: f32, sense_resistor: f32, vsense: bool) -> u8 {
    calc_cs(hold_current, sense_resistor, vsense)
}
