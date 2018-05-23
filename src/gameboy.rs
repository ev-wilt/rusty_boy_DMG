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
}



