use register_pair::*;

pub struct Cpu {

    // Register pairs
    reg_af: RegisterPair,
    reg_bc: RegisterPair,
    reg_de: RegisterPair,
    reg_hl: RegisterPair,

    // 16-bit registers
    reg_pc: u16,
    reg_sp: u16
}

impl Cpu {

    /// Default constructor.
    pub fn new() -> Cpu {
        Cpu {
            reg_af: RegisterPair::new(0x01B0),
            reg_bc: RegisterPair::new(0x0013),
            reg_de: RegisterPair::new(0x00D8),
            reg_hl: RegisterPair::new(0x014D),
            reg_pc: 0x0100,
            reg_sp: 0xFFFE
        }
    }

}