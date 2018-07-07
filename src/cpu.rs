use register_pair::*;
use memory_manager::*;
use display_manager::*;
use interrupt_handler::*;
use gamepad::*;
use std::rc::Rc;
use std::cell::RefCell;

pub struct Cpu {

    // Register pairs
    reg_af: RegisterPair,
    reg_bc: RegisterPair,
    reg_de: RegisterPair,
    reg_hl: RegisterPair,
    reg_sp: RegisterPair,

    // Program counter
    reg_pc: u16,

    // Memory manager
    memory_manager: Rc<RefCell<MemoryManager>>,

}

impl Cpu {

    /// Default constructor.
    pub fn new(memory_manager: Rc<RefCell<MemoryManager>>) -> Cpu {
        Cpu {
            reg_af: RegisterPair::new(0x01B0),
            reg_bc: RegisterPair::new(0x0013),
            reg_de: RegisterPair::new(0x00D8),
            reg_hl: RegisterPair::new(0x014D),
            reg_sp: RegisterPair::new(0xFFFE),
            reg_pc: 0x0100,
            memory_manager: memory_manager,
        }
    }

    /// Getter for the program counter.
    pub fn get_reg_pc(&self) -> u16 {
        self.reg_pc
    }

    /// Setter for the program counter.
    pub fn set_reg_pc(&mut self, reg_pc: u16) {
        self.reg_pc = reg_pc;
    } 

    /// Pushes a value onto the stack.
    pub fn stack_push(&mut self, val: u8) {
        let prev = self.reg_sp.get_pair();
        self.reg_sp.set_pair(prev - 1);
        self.memory_manager.borrow_mut().write_memory(self.reg_sp.get_pair(), val);
    }
}