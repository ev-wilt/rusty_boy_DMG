pub extern crate sdl2;

use cpu::*;
use memory_manager::*;
use interrupt_handler::*;
use display_manager::*;
use gamepad::*;

use std::cell::RefCell;
use std::rc::Rc;
use gameboy::sdl2::EventPump;

pub struct Gameboy {
    pub cpu: Cpu,
    pub memory_manager: Rc<RefCell<MemoryManager>>,
    pub interrupt_handler: InterruptHandler,
    pub display_manager: DisplayManager,
    pub gamepad: Gamepad,
    pub event_pump: EventPump
}

impl Gameboy {

    /// Default constructor.
    pub fn new() -> Gameboy {

        // SDL2 tools
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let event_pump = sdl_context.event_pump().unwrap();

        let memory_manager = Rc::new(RefCell::new(MemoryManager::new()));
        let cpu = Cpu::new(Rc::clone(&memory_manager));
        let interrupt_handler = InterruptHandler::new(Rc::clone(&memory_manager));
        let display_manager = DisplayManager::new(Rc::clone(&memory_manager), &video_subsystem);
        let gamepad = Gamepad::new(Rc::clone(&memory_manager));

        Gameboy {
            memory_manager: memory_manager,
            cpu: cpu,
            interrupt_handler: interrupt_handler,
            display_manager: display_manager,
            gamepad: gamepad,
            event_pump: event_pump
        }
    }

    /// Runs a single frame's worth of
    /// CPU cycles. Returns false when
    /// the emulation has stopped.
    pub fn step(&mut self) -> bool {
        let max_cycles = 69905;
        let mut cycles_per_step = 0;
        let event_pump = &mut self.event_pump;

        if !self.gamepad.poll_events(event_pump) {
            return false;
        }
        
        while cycles_per_step < max_cycles {
            let current_cycles = self.cpu.interpret_opcode();
            cycles_per_step += current_cycles;
            self.memory_manager.borrow_mut().update_timers(current_cycles);
            self.display_manager.update_display(current_cycles);
            self.interrupt_handler.check_interrupts(&mut self.cpu);
        }
        self.display_manager.draw_display();
        true
    }
}