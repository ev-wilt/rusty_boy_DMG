use interrupt_handler::*;
use std::rc::Rc;
use std::cell::RefCell;

pub struct Gamepad {
    interrupt_handler: InterruptHandler
}

impl Gamepad {
    pub fn new(interrupt_handler: InterruptHandler) -> Gamepad {
        Gamepad {
            interrupt_handler: interrupt_handler
        }
    }
}