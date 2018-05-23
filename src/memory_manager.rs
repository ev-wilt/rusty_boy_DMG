enum 

pub struct MemoryManager {
    memory: [u8; 0x10000]
}

impl MemoryManager {

    /// Default constructor.
    pub fn new() -> MemoryManager {
        let mut manager = MemoryManager {
            memory: [0; 0x10000]
        };

        // Initial startup sequence
        manager.memory[0xFF05] = 0x00; 
        manager.memory[0xFF06] = 0x00; 
        manager.memory[0xFF07] = 0x00; 
        manager.memory[0xFF10] = 0x80; 
        manager.memory[0xFF11] = 0xBF; 
        manager.memory[0xFF12] = 0xF3; 
        manager.memory[0xFF14] = 0xBF; 
        manager.memory[0xFF16] = 0x3F; 
        manager.memory[0xFF17] = 0x00; 
        manager.memory[0xFF19] = 0xBF; 
        manager.memory[0xFF1A] = 0x7F; 
        manager.memory[0xFF1B] = 0xFF; 
        manager.memory[0xFF1C] = 0x9F; 
        manager.memory[0xFF1E] = 0xBF; 
        manager.memory[0xFF20] = 0xFF; 
        manager.memory[0xFF21] = 0x00; 
        manager.memory[0xFF22] = 0x00; 
        manager.memory[0xFF23] = 0xBF; 
        manager.memory[0xFF24] = 0x77; 
        manager.memory[0xFF25] = 0xF3;
        manager.memory[0xFF26] = 0xF1; 
        manager.memory[0xFF40] = 0x91; 
        manager.memory[0xFF42] = 0x00; 
        manager.memory[0xFF43] = 0x00; 
        manager.memory[0xFF45] = 0x00; 
        manager.memory[0xFF47] = 0xFC; 
        manager.memory[0xFF48] = 0xFF; 
        manager.memory[0xFF49] = 0xFF; 
        manager.memory[0xFF4A] = 0x00; 
        manager.memory[0xFF4B] = 0x00; 
        manager.memory[0xFFFF] = 0x00; 
        manager
    }

    /// Writes byte to the given address in memory.
    pub fn write_memory(&mut self, address: u16, byte: u8) {
        // Prohibit writing to ROM
        if address < 0x8000 {
            panic!("Attempted to write 0x{:02X} at memory location 0x{:04X}, which is read-only.", byte, address);
        }

        // Shadow of work RAM
        else if address >= 0xE000 && address < 0xFE00 {
            self.memory[address as usize] = byte;
            self.write_memory(address - 0x2000, byte)
        }

        // Unusable memory
        else if address >= 0xFEA0 && address < 0xFEFF {
            panic!("Attempted to write 0x{:02X} at memory location 0x{:04X}, which is unsusable.", byte, address);
        }

        // Write to memory normally in all other cases
        else {
            self.memory[address as usize] = byte;
        }
    }
}