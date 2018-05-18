use std::io;
use std::fs::File;
use std::io::Read;

pub struct Core {

}

impl Core {
    pub fn new() -> Core {
        Core {

        }
    }

    /// Reads a game to a vector on success.
    pub fn load_rom(&mut self, location: &str) -> io::Result<Vec<u8>> {
        let mut rom = File::open(location)?;
        let mut buffer = Vec::new();
        rom.read_to_end(&mut buffer)?;
        Ok(buffer)
    }
}



