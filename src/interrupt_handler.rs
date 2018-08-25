use memory_manager::*;
use cpu::*;

use std::rc::Rc;
use std::cell::RefCell;

pub struct InterruptHandler {
    memory_manager: Rc<RefCell<MemoryManager>>
}

impl InterruptHandler {

    /// Default constructor.
    pub fn new(memory_manager: Rc<RefCell<MemoryManager>>) -> InterruptHandler {
        InterruptHandler {
            memory_manager: memory_manager
        }
    }

    /// Handles an interrupt for a given bit.
    pub fn handle_interrupt(&mut self, bit: u8, cpu: &mut Cpu) {
        if cpu.get_interrupts_enabled() {
            cpu.set_interrupts_enabled(false);
            let mut request_value = self.memory_manager.borrow_mut().read_memory(0xFF0F);
            request_value ^= 1 << bit;
            self.memory_manager.borrow_mut().write_memory(0xFF0F, request_value);

            // Push PC onto stack
            let pc = cpu.get_reg_pc();
            cpu.stack_push(pc);

            match bit {
                0 => cpu.set_reg_pc(0x40),    // V-Blank
                1 => cpu.set_reg_pc(0x48),    // LCD
                2 => cpu.set_reg_pc(0x50),    // Timer
                4 => cpu.set_reg_pc(0x60),    // Controller
                _ => panic!("Invalid bit given to interrupt handler: {}", bit)
            }
        }
        cpu.set_halted(false);
    }

    /// Checks if any interrupts need to be handled.
    pub fn check_interrupts(&mut self, cpu: &mut Cpu) {
        if cpu.get_interrupts_enabled() {
            let request_value = self.memory_manager.borrow_mut().read_memory(0xFF0F);
            let enabled_value = self.memory_manager.borrow_mut().read_memory(0xFFFF);

            if request_value > 0 {
                for i in 0..5 {

                    // Check if request and enabled registers 
                    // are set to 1
                    if (request_value & (1 << i)) >> i == 1 && (enabled_value & (1 << i)) >> i == 1 {
                        self.handle_interrupt(i, cpu);
                    }
                }
            }
        }
    }

    /// Requests an interrupt for the given bit,
    /// where bits 0-4 are the different interrupts.
    pub fn request_interrupt(&mut self, bit: u8) {
        let mut request_value = self.memory_manager.borrow_mut().read_memory(0xFF0F);
        request_value |= 1 << bit;
        self.memory_manager.borrow_mut().write_memory(0xFF0F, request_value);
    }

}