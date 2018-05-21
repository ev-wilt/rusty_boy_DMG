pub struct MemoryManager {
    memory: [u8; 0x10000]
}

impl MemoryManager {

    /// Default constructor.
    pub fn new() -> MemoryManager {
        let mut manager = MemoryManager {
            memory: [0; 0x10000]
        };
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

    
}