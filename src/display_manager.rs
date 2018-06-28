use memory_manager::*;
use interrupt_handler::*;
use std::rc::Rc;
use std::cell::RefCell;

pub struct DisplayManager {
    display: [[[u8; 3]; 144]; 160],
    remaining_cycles: i32,
    memory_manager: Rc<RefCell<MemoryManager>>,
    interrupt_handler: Rc<RefCell<InterruptHandler>>
}

impl DisplayManager {

    /// Default constructor.
    pub fn new(memory_manager: Rc<RefCell<MemoryManager>>, interrupt_handler: Rc<RefCell<InterruptHandler>>) -> DisplayManager {
        DisplayManager {
            display: [[[0; 3]; 144]; 160],
            remaining_cycles: 456,
            memory_manager: memory_manager,
            interrupt_handler: interrupt_handler
        }
    }

    /// Draws the current scanline.
    pub fn draw_scanline(&mut self) {
        let display_control = self.memory_manager.borrow_mut().read_memory(0xFF40);

        if display_control & 1 == 1 {
            // Render tiles
        }
        if display_control & (1 << 1) == 1 {
            // Render sprites
        }
    }

    /// Determines whether the scanline
    /// needs to be updated.
    pub fn update_display(&mut self, cycles: i32) {
        self.set_display_status();
        
        // Update remaining cycles only if 
        // the display is enabled
        if self.display_enabled() {
            self.remaining_cycles -= cycles;
        }
        else {
            return;
        }

        // Move to next scanline
        if self.remaining_cycles <= 0 {
            let next_scanline = self.memory_manager.borrow_mut().read_memory(0xFF44) + 1;
            self.memory_manager.borrow_mut().write_memory(0xFF44, next_scanline);
            self.remaining_cycles = 456;

            // V-Blank
            if next_scanline == 144 {
                self.interrupt_handler.borrow_mut().request_interrupt(0);
            }

            // Reset scanline
            else if next_scanline > 153 {
                self.memory_manager.borrow_mut().write_memory(0xFF44, 0);
            }

            else if next_scanline < 144 {
                self.draw_scanline();
            }
        }
    }

    
    pub fn set_display_status(&mut self) {
        let mut display_status = self.memory_manager.borrow_mut().read_memory(0xFF41);

        if !self.display_enabled() {
            self.remaining_cycles = 456;
            self.memory_manager.borrow_mut().write_memory(0xFF44, 0);
            display_status &= 0xFC;
            display_status |= 1;
            self.memory_manager.borrow_mut().write_memory(0xFF41, display_status);
            return;
        }

        let previous_mode = display_status & 0x3;
        let mut new_mode = 0;
        let current_scanline = self.memory_manager.borrow_mut().read_memory(0xFF44);
        let mut request_interrupt = false;

        // V-Blank
        if current_scanline >= 144 {
            new_mode = 1;
            
            // Set bit 0 to 1, bit 1 to 0
            display_status |= 1;
            display_status &= 0xFD;

        }
        else {
            let mode_2_min = 456 - 80;
            let mode_3_min = mode_2_min - 172;

            // Mode 2
            if self.remaining_cycles >= mode_2_min {
                new_mode = 2;

                // Set bit 0 to 0, bit 1 to 1
                display_status |= 1 << 1;
                display_status &= 0xFE;
                if display_status & (1 << 5) == 1 {
                    request_interrupt = true;
                }
            }

            // Mode 3
            else if self.remaining_cycles >= mode_3_min {
                new_mode = 2;
                
                // Set bit 0 to 1, bit 1 to 1
                display_status |= 3;
            }

            // Mode 0
            else {
                new_mode = 0;

                // Set bit 0 to 0, bit 1 to 0
                display_status &= 0xFC;
                if display_status & (1 << 3) == 1 {
                    request_interrupt = true;
                }
            }
        }

        if request_interrupt && (new_mode != previous_mode) {
            self.interrupt_handler.borrow_mut().request_interrupt(1);
        }

        if current_scanline == self.memory_manager.borrow_mut().read_memory(0xFF45) {
            
            display_status |= 1 << 2;
            if display_status & (1 << 6) == 1 {
                self.interrupt_handler.borrow_mut().request_interrupt(1);
            }
        }
        else {
            // Set bit 2 to 0
            display_status &= 0xFB;
        }
        self.memory_manager.borrow_mut().write_memory(0xFF41, display_status);
    }

    /// Tests bit 7 at 0xFF40 to determine if the 
    /// display is enabled.
    pub fn display_enabled(&mut self) -> bool {
        let display_bit = if (self.memory_manager.borrow_mut().read_memory(0xFF40) & 0x40) >> 6 == 1 {
            true
        }
        else {
            false
        };
        display_bit
    }
}