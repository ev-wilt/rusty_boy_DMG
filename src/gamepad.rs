use memory_manager::*;

use std::rc::Rc;
use std::cell::RefCell;
use sdl2::EventPump;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

pub struct Gamepad {
    memory_manager: Rc<RefCell<MemoryManager>>,
}

impl Gamepad {
    pub fn new(memory_manager: Rc<RefCell<MemoryManager>>) -> Gamepad {
        Gamepad {
            memory_manager: memory_manager,
        }
    }

    /// Returns an integer to identify 
    /// the keycode.
    pub fn resolve_key(&mut self, key: Keycode) -> Option<i32> {
        match key {
            Keycode::D => Some(0),
            Keycode::A => Some(1),
            Keycode::W => Some(2),
            Keycode::S => Some(3),
            Keycode::K => Some(4),
            Keycode::J => Some(5),
            Keycode::Space => Some(6),
            Keycode::Return => Some(7),
            _ => None
        }
    }

    /// Polls the current events in the
    /// event pump and return false
    /// when the quit event occurs.
    pub fn poll_events(&mut self, event_pump: &mut EventPump) -> bool {

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    return false;
                },
                Event::KeyDown { keycode: Some(keycode), repeat, .. } => {
                    if !repeat {
                        let key_val = self.resolve_key(keycode);
                        if key_val != None {
                            self.key_pressed(key_val.unwrap());
                        }
                    }
                },
                Event::KeyUp { keycode: Some(keycode), .. } => {
                    let key_val = self.resolve_key(keycode);
                    if key_val != None {
                        self.key_released(key_val.unwrap());
                    }                
                },
                _ => {}
            }
        }
        true
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
        let mut state_changed = true;

        if (self.memory_manager.borrow_mut().gamepad_state & (1 << key)) >> key == 0 {
            state_changed = false;
        }

        self.memory_manager.borrow_mut().gamepad_state ^= 1 << key;

        // Check if the key is a button or 
        // direction
        let key_is_button = if key > 3 { true } else { false };

        // Determine if an interrupt is needed
        let gamepad_reg = self.memory_manager.borrow_mut().read_memory(0xFF00);
        let mut will_request_interrupt = false;
        
        if key_is_button && ((gamepad_reg & (1 << 5)) >> 5) == 0 {
            will_request_interrupt = true;
        }
        else if !key_is_button && ((gamepad_reg & (1 << 4)) >> 4) == 0 {
            will_request_interrupt = true;
        }

        if will_request_interrupt && state_changed {
            self.memory_manager.borrow_mut().request_interrupt(4);
        }
    }
}