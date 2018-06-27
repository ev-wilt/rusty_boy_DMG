use register_pair::*;
use memory_manager::*;
use display_manager::*;
use interrupt_handler::*;

pub struct Cpu<'cpu> {

    // Register pairs
    reg_af: RegisterPair,
    reg_bc: RegisterPair,
    reg_de: RegisterPair,
    reg_hl: RegisterPair,
    reg_sp: RegisterPair,

    // Program counter
    reg_pc: u16,

    // Memory manager
    memory_manager: MemoryManager,

    // Display manager
    display_manager: DisplayManager<'cpu>,

    // Interrupt handler
    interrupt_handler: InterruptHandler<'cpu>
}

impl<'cpu> Cpu<'cpu> {

    /// Default constructor.
    pub fn new() -> Cpu<'cpu> {
        let memory_manager = MemoryManager::new();
        let mut cpu = Cpu {
            reg_af: RegisterPair::new(0x01B0),
            reg_bc: RegisterPair::new(0x0013),
            reg_de: RegisterPair::new(0x00D8),
            reg_hl: RegisterPair::new(0x014D),
            reg_sp: RegisterPair::new(0xFFFE),
            reg_pc: 0x0100,
            memory_manager: memory_manager,
            display_manager: DisplayManager::new(),
            interrupt_handler: InterruptHandler::new()
        };
        let interrupt_handler = InterruptHandler::new();

        cpu.interrupt_handler.memory_manager = &mut cpu.memory_manager;
        cpu
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
        let interrupt_handler = &mut self.interrupt_handler;
        self.memory_manager.update_timers(cycles, interrupt_handler);
    }

    /// Pushes a value onto the stack.
    pub fn stack_push(&mut self, val: u8) {
        let prev = self.reg_sp.get_pair();
        self.reg_sp.set_pair(prev - 1);
        self.memory_manager.write_memory(self.reg_sp.get_pair(), val);
    }
}