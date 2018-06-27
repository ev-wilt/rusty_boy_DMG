use gameboy::*;

pub struct Core<'co> {
    pub gameboy: Gameboy<'co>
}

impl<'co> Core<'co> {

    /// Default constructor.
    pub fn new() -> Core<'co> {
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