use cartridge::*;
use cpu::*;

pub struct Gameboy {
    pub cartridge: Cartridge,
    pub cpu: Cpu
}

impl Gameboy {

    /// Default constructor.
    pub fn new() -> Gameboy {
        Gameboy {
            cartridge: Cartridge::new(),
            cpu: Cpu::new()
        }
    }
}



