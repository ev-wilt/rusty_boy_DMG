use gameboy::*;

pub struct Core {
    pub gameboy: Gameboy
}

impl Core {
    pub fn new() -> Core {
        Core {
            gameboy: Gameboy::new()
        }
    }
}



