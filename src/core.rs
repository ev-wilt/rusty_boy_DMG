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

    /// Executes a single frames' worth of instructions.
    pub fn update(&mut self) {
        self.gameboy.step();
        // Render display
    }
}