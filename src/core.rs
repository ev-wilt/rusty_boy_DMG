use gameboy::*;

pub struct Core {
    pub gameboy: Gameboy
}

impl Core {

    /// Default constructor.
    pub fn new() -> Core {
        Core {
            gameboy: Gameboy::new()
        }
    }
}



