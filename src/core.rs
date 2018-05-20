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
    pub fn update() {
        let max_cycles = 69905;
        let mut current_cycles = 0;

        while current_cycles < max_cycles {
            // Set current cycles and execute instruction
            // Update timers
            // Update graphics
            // Do interrupts
        }
        // Render display
    }
}



