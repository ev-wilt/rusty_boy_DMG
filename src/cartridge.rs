use std::io;
use std::fs::File;
use std::io::Read;
use std::env;

enum BankingType {
    NoBanking,
    MBC1,
    MBC2
}

pub struct Cartridge {
    rom: Vec<u8>,
    ram_banks: [u8; 0x8000],
    banking_type: BankingType,
    current_rom_bank: u8,
    current_ram_bank: u8
}

impl Cartridge {

    /// Default constructor.
    pub fn new() -> Cartridge {
        let mut cartridge = Cartridge {
            rom: Vec::new(),
            ram_banks: [0; 0x8000],
            banking_type: BankingType::NoBanking,
            current_rom_bank: 1,
            current_ram_bank: 0
        };

        // Set rom to bytes from file
        let args: Vec<String> = env::args().collect();
        let rom = cartridge.read_rom(&args[1]);
        let rom = match rom {
            Ok(rom) => rom,
            Err(e) => panic!("{}", e),
        };
        cartridge.rom = rom;

        // Set rom banking type
        match cartridge.rom[0x147] {
            0 => cartridge.banking_type = BankingType::NoBanking,
            1 | 2 | 3 => cartridge.banking_type = BankingType::MBC1,
            4 | 5 | 6 => cartridge.banking_type = BankingType::MBC2,
            _ => panic!("Banking type of the ROM is currently not supported. Value at 0x147 was 0x{:02X}", cartridge.rom[0x147])
        }
        cartridge
    }

    /// Reads a rom file's bytes to a vector on success.
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

    /// Returns the byte in rom at a given address.
    pub fn get_rom(&mut self, address: u32) -> u8 {
        self.rom[address as usize]
    }

    /// Returns the byte in a ram bank at a given address.
    pub fn get_ram(&mut self, address: u16) -> u8 {
        self.ram_banks[address as usize]
    }

    /// Getter for the current rom bank.
    pub fn get_current_rom_bank(&mut self) -> u8 {
        self.current_rom_bank
    }

    /// Getter for the current ram bank.
    pub fn get_current_ram_bank(&mut self) -> u8 {
        self.current_ram_bank
    }
}