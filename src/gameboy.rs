use cartridge::*;

pub struct Gameboy {
    pub cartridge: Cartridge
}

impl Gameboy {
    pub fn new() -> Gameboy {
        Gameboy {
            cartridge: Cartridge::new()
        }
    }
}



