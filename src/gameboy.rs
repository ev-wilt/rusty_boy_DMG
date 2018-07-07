use cpu::*;
use memory_manager::*;
use interrupt_handler::*;
use display_manager::*;
use gamepad::*;

use std::cell::RefCell;
use std::rc::Rc;

pub struct Gameboy {
    pub cpu: Cpu,
    pub memory_manager: Rc<RefCell<MemoryManager>>,
    pub interrupt_handler: InterruptHandler,
    pub display_manager: DisplayManager
}

impl Gameboy {

    /// Default constructor.
    pub fn new() -> Gameboy {
        let memory_manager = Rc::new(RefCell::new(MemoryManager::new()));
        let cpu = Cpu::new(Rc::clone(&memory_manager));
        let interrupt_handler = InterruptHandler::new(Rc::clone(&memory_manager));
        let display_manager = DisplayManager::new(Rc::clone(&memory_manager), InterruptHandler::new(Rc::clone(&memory_manager)));
        Gameboy {
            memory_manager: memory_manager,
            cpu: cpu,
            interrupt_handler: interrupt_handler,
            display_manager: display_manager
        }
    }

    /// Runs a single frame's worth of
    /// CPU cycles.
    pub fn step(&mut self) {
        let max_cycles = 69905;
        let cycles_per_step = 0;

        while cycles_per_step < max_cycles {
            let current_cycles = 0;
            // Set current cycles and execute instruction
            self.memory_manager.borrow_mut().update_timers(current_cycles, &mut self.interrupt_handler);
            // Update graphics
            // Do interrupts
        }
    }
}