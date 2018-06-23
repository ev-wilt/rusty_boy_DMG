use memory_manager::*;
use interrupt_handler::*;

pub struct DisplayManager {
    display: [[[u8; 3]; 144]; 160],
    remaining_cycles: i32
}

impl DisplayManager {

    /// Default constructor.
    pub fn new() -> DisplayManager {
        DisplayManager {
            display: [[[0; 3]; 144]; 160],
            remaining_cycles: 456
        }
    }


    pub fn update_display(&mut self, cycles: i32, memory_manager: &mut MemoryManager, interrupt_handler: &mut InterruptHandler) {
        self.set_display_status(memory_manager, interrupt_handler);
        
        // Update remaining cycles only if 
        // the display is enabled
        if self.display_enabled(memory_manager) {
            self.remaining_cycles -= cycles;
        }
        else {
            return;
        }

        if self.remaining_cycles <= 0 {
            let next_scanline = memory_manager.read_memory(0xFF44) + 1;
            memory_manager.write_memory(0xFF44, next_scanline);
            self.remaining_cycles = 456;

            // V-Blank
            if next_scanline == 144 {
                interrupt_handler.request_interrupt(0, memory_manager);
            }

            // Reset scanline
            else if next_scanline > 153 {
                memory_manager.write_memory(0xFF44, 0);
            }

            else if next_scanline < 144 {
                // Draw scanline
            }
        }
    }

    
    pub fn set_display_status(&mut self, memory_manager: &mut MemoryManager, interrupt_handler: &mut InterruptHandler) {

    }

    /// Tests byte at 0xFF40 to determine if the 
    /// display is enabled.
    pub fn display_enabled(&mut self, memory_manager: &mut MemoryManager) -> bool {
        let display_bit = if (memory_manager.read_memory(0xFF40) & 0x40) >> 6 == 1 {
            true
        }
        else {
            false
        };
        display_bit
    }
}