use register::*;
use register_pair::*;

pub fn ld_reg_pair_reg_pair(src: &mut RegisterPair, dest: &mut RegisterPair) {
    dest.set_pair(src.get_pair());
}

pub fn ld_u16_reg_pair(src: u16, dest: &mut RegisterPair) {
    dest.set_pair(src);
}

pub fn ld_reg_reg(src: &mut Register, dest: &mut Register) {
    dest.set_val(src.get_val());
}

pub fn ld_u8_reg(src: u8, dest: &mut Register) {
    dest.set_val(src);
}

