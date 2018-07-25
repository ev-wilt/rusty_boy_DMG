use register_pair::*;

pub fn test_bit(byte: u8, bit: u8) -> bool {
    if (byte & (1 << bit)) >> bit == 1 {
        return true;
    }
    false
}

pub fn ld_reg_pair_reg_pair(src: &mut RegisterPair, dest: &mut RegisterPair) {
    dest.set_pair(src.get_pair());
}

pub fn inc_reg_pair(dest: &mut RegisterPair) {
    let prev_pair = dest.get_pair();
    dest.set_pair(prev_pair.wrapping_add(1));
}

pub fn dec_reg_pair(dest: &mut RegisterPair) {
    let prev_pair = dest.get_pair();
    dest.set_pair(prev_pair.wrapping_sub(1));
}

pub fn ld_u16_reg_pair(src: u16, dest: &mut RegisterPair) {
    dest.set_pair(src);
}

pub fn ld_u8_reg(src: u8, dest: &mut u8) {
    *dest = src;
}