use register_pair::*;
use memory_manager::*;
use display_manager::*;
use interrupt_handler::*;
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

    // Display manager
    display_manager: DisplayManager,

    // Interrupt handler
    interrupt_handler: Rc<RefCell<InterruptHandler>>
}

impl Cpu {

    /// Default constructor.
    pub fn new() -> Cpu {
        let memory_manager = Rc::new(RefCell::new(MemoryManager::new()));
        let interrupt_handler = Rc::new(RefCell::new(InterruptHandler::new(Rc::clone(&memory_manager))));
        let display_manager = DisplayManager::new(Rc::clone(&memory_manager), Rc::clone(&interrupt_handler));
        Cpu {
            reg_af: RegisterPair::new(0x01B0),
            reg_bc: RegisterPair::new(0x0013),
            reg_de: RegisterPair::new(0x00D8),
            reg_hl: RegisterPair::new(0x014D),
            reg_sp: RegisterPair::new(0xFFFE),
            reg_pc: 0x0100,
            memory_manager: memory_manager,
            display_manager: display_manager,
            interrupt_handler: interrupt_handler
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

    /// Wrapper function for the memory manager's
    /// update timer method.
    pub fn update_timers(&mut self, cycles: i32) {
        let interrupt_handler = &self.interrupt_handler;
        self.memory_manager.borrow_mut().update_timers(cycles, interrupt_handler);
    }

    /// Pushes a value onto the stack.
    pub fn stack_push(&mut self, val: u8) {
        let prev = self.reg_sp.get_pair();
        self.reg_sp.set_pair(prev - 1);
        self.memory_manager.borrow_mut().write_memory(self.reg_sp.get_pair(), val);
    }
}