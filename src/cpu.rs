use register_pair::*;
use memory_manager::*;

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
    memory_manager: MemoryManager,

    // Master interrupt switch
    interrupts_enabled: bool
}

impl Cpu {

    /// Default constructor.
    pub fn new() -> Cpu {
        Cpu {
            reg_af: RegisterPair::new(0x01B0),
            reg_bc: RegisterPair::new(0x0013),
            reg_de: RegisterPair::new(0x00D8),
            reg_hl: RegisterPair::new(0x014D),
            reg_sp: RegisterPair::new(0xFFFE),
            reg_pc: 0x0100,
            memory_manager: MemoryManager::new(),
            interrupts_enabled: false
        }
    }

    /// Wrapper function for the memory manager's
    /// update timer method.
    pub fn update_timers(&mut self, cycles: i32) {
        self.memory_manager.update_timers(cycles);
    }

    /// Requests an interrupt for the given bit,
    /// where bits 0-4 are the different interrupts.
    pub fn request_interrupt(&mut self, bit: u8) {
        let mut request_value = self.memory_manager.read_memory(0xFF0F);
        request_value |= 1 << bit;
        self.memory_manager.write_memory(0xFF0F, request_value);
    }

    /// Pushes a value onto the stack.
    pub fn stack_push(&mut self, val: u8) {
        let prev = self.reg_sp.get_pair();
        self.reg_sp.set_pair(prev - 1);
        self.memory_manager.write_memory(self.reg_sp.get_pair(), val);
    }

    /// Handles an interrupt for a given bit.
    pub fn handle_interrupt(&mut self, bit: u8) {
        self.interrupts_enabled = false;
        let mut request_value = self.memory_manager.read_memory(0xFF0F);
        request_value ^= 1 << bit;
        self.memory_manager.write_memory(0xFF0F, request_value);

        // Push PC onto stack
        let pc = self.reg_pc;
        self.stack_push(pc as u8);

        match bit {
            0 => self.reg_pc = 0x40,    // V-Blank
            1 => self.reg_pc = 0x48,    // LCD
            2 => self.reg_pc = 0x50,    // Timer
            4 => self.reg_pc = 0x60,    // Controller
            _ => panic!("Invalid bit given to interrupt handler: {}", bit)
        }
    }

    /// Checks if any interrupts need to be handled.
    pub fn check_interrupts(&mut self) {
        if self.interrupts_enabled {
            let request_value = self.memory_manager.read_memory(0xFF0F);
            let enabled_value = self.memory_manager.read_memory(0xFFFF);

            if request_value > 0 {
                for i in 0..5 {

                    // Check if request and enabled registers 
                    // are set to 1
                    if (request_value & (1 << i)) >> i == 1 && (enabled_value & (1 << i)) >> i == 1 {
                        self.handle_interrupt(i);
                    }
                }
            }
        }
    }
}