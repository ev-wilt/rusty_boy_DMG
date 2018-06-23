use cpu::*;

pub struct Gameboy {
    pub cpu: Cpu
}

impl Gameboy {

    /// Default constructor.
    pub fn new() -> Gameboy {
        Gameboy {
            cpu: Cpu::new()
        }
    }

    /// Runs a single frame's worth of
    /// CPU cycles.
    pub fn step(&mut self) {
        let max_cycles = 69905;
        let mut cycles_per_step = 0;

        while cycles_per_step < max_cycles {
            let current_cycles = 0;
            // Set current cycles and execute instruction
            self.cpu.update_timers(current_cycles);
            // Update graphics
            // Do interrupts
        }
    }
}