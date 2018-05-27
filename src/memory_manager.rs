use cartridge::*;

static TIMER: u16 = 0xFF05;
static TIMER_MODULATOR: u16 = 0xFF06;
static TIMER_CONTROLLER: u16 = 0xFF07;

pub struct MemoryManager {
    cartridge: Cartridge,
    memory: [u8; 0x10000],
    timer_counter: i32,
    div_counter: i32
}

impl MemoryManager {

    /// Default constructor.
    pub fn new() -> MemoryManager {
        let mut manager = MemoryManager {
            cartridge: Cartridge::new(),
            memory: [0; 0x10000],
            timer_counter: 1024,
            div_counter: 0
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

    /// Returns the clock frequency using the
    /// first 2 bits of the timer controller.
    pub fn get_frequency(&mut self) -> u8 {
        self.read_memory(TIMER_CONTROLLER) & 0x03
    }

    /// Sets the clocks frequency.
    pub fn set_frequency(&mut self) {
        let new_frequency = self.get_frequency();
        match new_frequency {
            0 => self.timer_counter = 1024,
            1 => self.timer_counter = 16,
            2 => self.timer_counter = 64,
            3 => self.timer_counter = 256,
            _ => panic!("Impossible value for frequency: {}", new_frequency)
        }
    }

    /// Writes byte to the given address in memory.
    pub fn write_memory(&mut self, address: u16, byte: u8) {
        
        // Banking
        if address < 0x8000 {
            self.cartridge.manage_banking(address, byte);
        }

        // Writing to RAM bank
        else if address >= 0xA000 && address < 0xC000 {
            if self.cartridge.ram_write_enabled {
                let shifted_address: u16 = address - 0xA000;
                let ram_bank = self.cartridge.get_current_ram_bank();
                self.cartridge.set_ram(shifted_address + (ram_bank as u16 * 0x2000), byte);
            }
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

        // Updating frequency
        else if address == TIMER_CONTROLLER {
            let frequency = self.get_frequency();
            self.memory[TIMER_CONTROLLER as usize] = byte;
            let new_frequency = self.get_frequency();

            if frequency != new_frequency {
                self.set_frequency();
            }
        }

        // Write to memory normally in all other cases
        else {
            self.memory[address as usize] = byte;
        }
    }

    /// Reads a byte from the given address in memory.
    pub fn read_memory(&mut self, address: u16) -> u8 {
        
        // Reading ROM bank
        if address >= 0x4000 && address <= 0x7FFF {
            let shifted_address: u16 = address - 0x4000;
            let rom_bank = self.cartridge.get_current_rom_bank();
            return self.cartridge.get_rom((shifted_address + (rom_bank as u16 * 0x4000)) as u32);
        }

        // Reading RAM bank
        if address >= 0xA000 && address <= 0xBFFF {
            let shifted_address: u16 = address - 0xA000;
            let ram_bank = self.cartridge.get_current_ram_bank();
            return self.cartridge.get_ram(shifted_address + (ram_bank as u16 * 0x2000));
        }

        // Return byte normally otherwise
        self.memory[address as usize]
    }

    /// Updates the divider register every 256 cycles.
    pub fn update_div_register(&mut self, cycles: i32) {
        self.div_counter += cycles;
        if self.div_counter >= 255 {
            self.div_counter = 0;
            self.memory[0xFF04] += 1;
        }
    }

    /// Returns whether the clock has been enabled.
    pub fn clock_enabled(&mut self) -> bool {
        if (self.read_memory(TIMER_CONTROLLER) & 0x02) == 1 {
            return true;
        }
        false
    }

    /// Updates the timers based on the current
    /// amount of CPU cycles.
    pub fn update_timers(&mut self, cycles: i32) {
        self.update_div_register(cycles);

        // Update timer only if clock is enabled
        if self.clock_enabled() {
            self.timer_counter -= cycles;

            if self.timer_counter <= 0 {
                self.set_frequency();

                if self.read_memory(TIMER) == 0xFF {
                    let modulator = self.read_memory(TIMER_MODULATOR);
                    self.write_memory(TIMER, modulator);
                    // Request interrupt (2)
                }
                else {
                    let increment_timer = self.read_memory(TIMER);
                    self.write_memory(TIMER, increment_timer);
                }
            }
        }
    }
}