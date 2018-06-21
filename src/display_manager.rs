use memory_manager::*;
use interrupt_handler::*;

pub struct DisplayManager {
    display: [[[u8; 3]; 144]; 160]
}

impl DisplayManager {

    /// Default constructor.
    pub fn new() -> DisplayManager {
        DisplayManager {
            display: [[[0; 3]; 144]; 160]
        }
    }


    pub fn update_display(&mut self, interrupt_handler: &mut InterruptHandler) {

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