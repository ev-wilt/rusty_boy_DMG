pub struct Cpu {

    // 8-bit registers
    reg_a: u8,
    reg_b: u8,
    reg_c: u8,
    reg_d: u8,
    reg_e: u8,
    reg_f: u8,
    reg_h: u8,
    reg_l: u8,

    // 16-bit registers
    reg_pc: u16,
    reg_sp: u16
}

impl Cpu {

    /// Default constructor.
    pub fn new() -> Cpu {
        Cpu {
            reg_a: 0x01,
            reg_b: 0x00,
            reg_c: 0x13,
            reg_d: 0x00,
            reg_e: 0xD8,
            reg_f: 0xB0,
            reg_h: 0x01,
            reg_l: 0x4D,
            reg_pc: 0x0100,
            reg_sp: 0xFFFE
        }
    }


    /// Getter for a register pair.
    pub fn reg_pair(&mut self, reg_x: u8, reg_y: u8) -> u16 {
        return (reg_x as u16) << 8 | reg_y as u16;
    }

}