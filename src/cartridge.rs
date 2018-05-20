use std::io;
use std::fs::File;
use std::io::Read;
use std::env;

pub struct Cartridge {
    pub rom: Vec<u8>
}

impl Cartridge {

    /// Default constructor.
    pub fn new() -> Cartridge {
        let args: Vec<String> = env::args().collect();
        let mut cartridge = Cartridge {
            rom: Vec::new()
        };
        let rom = cartridge.read_rom(&args[1]);
        let rom = match rom {
            Ok(rom) => rom,
            Err(e) => panic!("{}", e),
        };
        cartridge.rom = rom;
        return cartridge;
    }

    /// Reads a game to a vector on success.
    pub fn read_rom(&mut self, location: &str) -> io::Result<Vec<u8>> {
        let mut rom = File::open(location)?;
        let mut buffer = Vec::new();
        rom.read_to_end(&mut buffer)?;
        
        // Panic if ROM has more bytes than possible
        // or is amount of bytes is not a power of two
        if buffer.len() > 0x200000 || (buffer.len() & (buffer.len() - 1)) != 0 {
            panic!("Invalid ROM size, {} bytes", buffer.len());
        }
        Ok(buffer)
    }
}