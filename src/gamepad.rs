use interrupt_handler::*;
use memory_manager::*;

use std::rc::Rc;
use std::cell::RefCell;

pub struct Gamepad {
    interrupt_handler: InterruptHandler,
    memory_manager: Rc<RefCell<MemoryManager>>
}

impl Gamepad {
    pub fn new(memory_manager: Rc<RefCell<MemoryManager>>, interrupt_handler: InterruptHandler) -> Gamepad {
        Gamepad {
            interrupt_handler: interrupt_handler,
            memory_manager: memory_manager
        }
    }

    /// Updates the gamepad's state when
    /// a key is released.
    pub fn key_released(&mut self, key: i32) {
        let new_state = self.memory_manager.borrow_mut().gamepad_state | (1 << key);
        self.memory_manager.borrow_mut().gamepad_state = new_state;
    }

    /// Updates the gamepad state and requests
    /// the necessary interrupt when a key is pressed.
    pub fn key_pressed(&mut self, key: i32) {

        // Check if the key was not set already
        let mut state_changed = false;

        if self.memory_manager.borrow_mut().gamepad_state & (1 << key) >> key == 0 {
            state_changed = true;
        }

        self.memory_manager.borrow_mut().gamepad_state ^= 1 << key;

        // Check if the key is a button or 
        // direction
        let mut key_is_button = false;

        if key > 3 {
            key_is_button = true;
        }

        // Determine if an interrupt is needed
        let gamepad_reg = self.memory_manager.borrow_mut().read_memory(0xFF00);
        let mut will_request_interrupt = false;
        
        if key_is_button && (gamepad_reg & (1 << 5) >> 5) == 0 {
            will_request_interrupt = true;
        }
        else if !key_is_button && (gamepad_reg & (1 << 4) >> 4) == 0 {
            will_request_interrupt = true;
        }

        if will_request_interrupt && !state_changed {
            self.interrupt_handler.request_interrupt(4);
        }
    }
}