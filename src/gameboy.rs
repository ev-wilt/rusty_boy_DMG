use cartridge::*;

pub struct Gameboy {
    pub cartridge: Cartridge
}

impl Gameboy {

    /// Default constructor.
    pub fn new() -> Gameboy {
        Gameboy {
            cartridge: Cartridge::new()
        }
    }
}



