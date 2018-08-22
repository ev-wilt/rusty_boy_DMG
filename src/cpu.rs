use register_pair::*;
use memory_manager::*;
use instructions::*;

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

    // Master interrupt switch
    interrupts_enabled: bool,

    halted: bool,
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
            interrupts_enabled: false,
            halted: false
        }
    }

    /// Returns the next byte in memory.
    pub fn get_byte(&mut self) -> u8 {
        let byte = self.memory_manager.borrow_mut().read_memory(self.reg_pc);
        self.reg_pc += 1;
        byte
    }

    /// Returns the next word in memory.
    pub fn get_word(&mut self) -> u16 {
        let byte_lo = self.memory_manager.borrow_mut().read_memory(self.reg_pc);
        let byte_hi = self.memory_manager.borrow_mut().read_memory(self.reg_pc + 1);
        let word = ((byte_hi as u16) << 8) | (byte_lo as u16);
        self.reg_pc += 2;
        word
    }

    /// Getter for the program counter.
    pub fn get_reg_pc(&mut self) -> u16 {
        self.reg_pc
    }

    /// Setter for the program counter.
    pub fn set_reg_pc(&mut self, reg_pc: u16) {
        self.reg_pc = reg_pc;
    } 

    /// Setter for the halted switch.
    pub fn set_halted(&mut self, halted: bool) {
        self.halted = halted;
    }

    /// Getter for the interrupt switch.
    pub fn get_interrupts_enabled(&mut self) -> bool {
        self.interrupts_enabled
    }

    /// Setter for the interrupt switch.
    pub fn set_interrupts_enabled(&mut self, interrupts_enabled: bool) {
        self.interrupts_enabled = interrupts_enabled;
    }

    /// Pushes a word onto the stack.
    pub fn stack_push(&mut self, val: u16) {
        let prev = self.reg_sp.get_pair();
        let val_lo = (val >> 8) as u8;
        let val_hi = (val & 0xFF) as u8;
        self.reg_sp.set_pair(prev - 1);
        self.memory_manager.borrow_mut().write_memory(self.reg_sp.get_pair(), val_hi);
        self.reg_sp.set_pair(prev - 2);
        self.memory_manager.borrow_mut().write_memory(self.reg_sp.get_pair(), val_lo);
    }

    /// Pops a word off the stack.
    pub fn stack_pop(&mut self) -> u16 {
        let prev = self.reg_sp.get_pair();
        let mut word = (self.memory_manager.borrow_mut().read_memory(self.reg_sp.get_pair()) as u16) << 8;
        word |= self.memory_manager.borrow_mut().read_memory(self.reg_sp.get_pair() + 1) as u16;
        self.reg_sp.set_pair(prev + 2);
        word
    } 

    /// Calls a subroutine at a given address.
    pub fn call_routine(&mut self, address: u16) {
        let pc = self.reg_pc;
        self.stack_push(pc);
        self.reg_pc = address;
    }

    /// Updates the bit for the zero flag.
    pub fn update_zero_flag(&mut self, result: bool) {
        if result {
            set_bit(&mut self.reg_af.lo, 7);
        }
        else {
            reset_bit(&mut self.reg_af.lo, 7);
        }
    }

    /// Updates the bit for the subtract flag.
    pub fn update_subtract_flag(&mut self, sub_occurred: bool) {
        if sub_occurred {
            set_bit(&mut self.reg_af.lo, 6);
        }
        else {
            reset_bit(&mut self.reg_af.lo, 6);
        }
    }

    /// Updates the bit for the half carry flag.
    pub fn update_half_carry_flag(&mut self, half_carry_occurred: bool) {
        if half_carry_occurred {
            set_bit(&mut self.reg_af.lo, 5);
        }
        else {
            reset_bit(&mut self.reg_af.lo, 5);
        }
    }

    /// Updates the bit for the carry flag.
    pub fn update_carry_flag(&mut self, carry_occurred: bool) {
        if carry_occurred {
            set_bit(&mut self.reg_af.lo, 4);
        }
        else {
            reset_bit(&mut self.reg_af.lo, 4);
        }
    }

    /// Moves the PC and executes the next opcode,
    /// then returns the number of cycles it 
    /// took.
    pub fn interpret_opcode(&mut self) -> i32 {

        // Don't run if halted
        if self.halted {
            return 4;
        }

        // println!("BC:\t0x{:04X}\nDE:\t0x{:04X}\nHL:\t0x{:04X}\nAF:\t0x{:04X}\nSP:\t0x{:04X}\nPC:\t0x{:04X}\n", self.reg_bc.get_pair(), self.reg_de.get_pair(), self.reg_hl.get_pair(), self.reg_af.get_pair(), self.reg_sp.get_pair(), self.reg_pc);
        let opcode = self.memory_manager.borrow_mut().read_memory(self.reg_pc);
        self.reg_pc += 1;

        // println!("Opcode: 0x{:02X}", opcode);
        match opcode {
            0x00 => { /* NOP */ 4 },
            0x01 => { ld_u16_reg_pair(self.get_word(), &mut self.reg_bc); 12 },
            0x02 => { self.memory_manager.borrow_mut().write_memory(self.reg_bc.get_pair(), self.reg_af.hi); 8 },
            0x03 => { inc_reg_pair(&mut self.reg_bc); 8 },
            0x04 => { 
                let mut b = self.reg_bc.hi;
                self.inc_u8(&mut b);
                self.reg_bc.hi = b;
                4
            },
            0x05 => { 
                let mut b = self.reg_bc.hi;
                self.dec_u8(&mut b);
                self.reg_bc.hi = b;
                4
            },
            0x06 => { ld_u8_reg(self.get_byte(), &mut self.reg_bc.hi); 8 },
            0x07 => {
                let mut a = self.reg_af.hi;
                self.rlc_u8(&mut a);
                self.reg_af.hi = a;
                4
            },
            0x08 => { 
                let address = self.get_word();
                self.memory_manager.borrow_mut().write_memory(address, self.reg_sp.lo);
                self.memory_manager.borrow_mut().write_memory(address + 1, self.reg_sp.hi);
                20
            },
            0x09 => {
                let mut bc = self.reg_bc.get_pair();
                self.add_u16_hl(&mut bc);
                self.reg_bc.set_pair(bc);
                8
            },
            0x0A => { self.reg_af.hi = self.memory_manager.borrow_mut().read_memory(self.reg_bc.get_pair()); 8 },
            0x0B => { 
                let val = self.reg_bc.get_pair();
                self.reg_bc.set_pair(val - 1);
                8
            },
            0x0C => { 
                let mut c = self.reg_bc.lo;
                self.inc_u8(&mut c);
                self.reg_bc.lo = c;
                4
            },
            0x0D => { 
                let mut c = self.reg_bc.lo;
                self.dec_u8(&mut c);
                self.reg_bc.lo = c;
                4
            },
            0x0E => { ld_u8_reg(self.get_byte(), &mut self.reg_bc.lo); 8 },
            0x0F => {
                let mut a = self.reg_af.hi;
                self.rrc_u8(&mut a);
                self.reg_af.hi = a;
                4
            },
            0x10 => { /* STOP */  4 },
            0x11 => { ld_u16_reg_pair(self.get_word(), &mut self.reg_de); 12 },
            0x12 => { self.memory_manager.borrow_mut().write_memory(self.reg_de.get_pair(), self.reg_af.hi); 8 },
            0x13 => { inc_reg_pair(&mut self.reg_de); 8 },
            0x14 => { 
                let mut d = self.reg_de.hi;
                self.inc_u8(&mut d);
                self.reg_de.hi = d;
                4
            },
            0x15 => { 
                let mut d = self.reg_de.hi;
                self.dec_u8(&mut d);
                self.reg_de.hi = d;
                4
            },
            0x16 => { ld_u8_reg(self.get_byte(), &mut self.reg_de.hi); 8 },
            0x17 => {
                let mut val = self.reg_af.hi;
                self.rl_u8(&mut val);
                self.reg_af.hi = val;
                4
            },
            0x18 => { self.reg_pc = ((self.get_byte() as i8) as i32 + ((self.reg_pc as u32) as i32)) as u16; 12 },
            0x19 => {
                let mut de = self.reg_de.get_pair();
                self.add_u16_hl(&mut de);
                self.reg_de.set_pair(de);
                8
            },
            0x1A => { self.reg_af.hi = self.memory_manager.borrow_mut().read_memory(self.reg_de.get_pair()); 8 },
            0x1B => { 
                let val = self.reg_de.get_pair();
                self.reg_de.set_pair(val - 1);
                8
            },
            0x1C => { 
                let mut e = self.reg_de.lo;
                self.inc_u8(&mut e);
                self.reg_de.lo = e;
                4
            },
            0x1D => { 
                let mut e = self.reg_de.lo;
                self.dec_u8(&mut e);
                self.reg_de.lo = e;
                4
            },
            0x1E => { ld_u8_reg(self.get_byte(), &mut self.reg_de.lo); 8 },
            0x1F => {
                let mut a = self.reg_af.hi;
                self.rr_u8(&mut a);
                self.reg_af.hi = a;
                4
            },
            0x20 => {
                if !test_bit(self.reg_af.lo, 7) {
                    self.reg_pc = ((self.get_byte() as i8) as i32 + ((self.reg_pc as u32) as i32)) as u16;
                    12
                }
                else {
                    self.reg_pc += 1;
                    8
                }
            },
            0x21 => { ld_u16_reg_pair(self.get_word(), &mut self.reg_hl); 12 },
            0x22 => {
                self.memory_manager.borrow_mut().write_memory(self.reg_hl.get_pair(), self.reg_af.hi);
                inc_reg_pair(&mut self.reg_hl);
                8
            },
            0x23 => { inc_reg_pair(&mut self.reg_hl); 8 },
            0x24 => { 
                let mut h = self.reg_hl.hi;
                self.inc_u8(&mut h);
                self.reg_hl.hi = h;
                4
            },
            0x25 => { 
                let mut h = self.reg_hl.hi;
                self.dec_u8(&mut h);
                self.reg_hl.hi = h;
                4
            },
            0x26 => { ld_u8_reg(self.get_byte(), &mut self.reg_hl.hi); 8 },
            0x27 => { self.daa(); 4 },
            0x28 => {
                if test_bit(self.reg_af.lo, 7) {
                    self.reg_pc = ((self.get_byte() as i8) as i32 + ((self.reg_pc as u32) as i32)) as u16;
                    12
                }
                else {
                    self.reg_pc += 1;
                    8
                }
            },
            0x29 => {
                let mut hl = self.reg_hl.get_pair();
                self.add_u16_hl(&mut hl);
                self.reg_hl.set_pair(hl);
                8
            },
            0x2A => {
                self.reg_af.hi = self.memory_manager.borrow_mut().read_memory(self.reg_hl.get_pair());
                inc_reg_pair(&mut self.reg_hl);
                8
            },
            0x2B => { 
                let val = self.reg_hl.get_pair();
                self.reg_hl.set_pair(val - 1);
                8
            },
            0x2C => { 
                let mut l = self.reg_hl.lo;
                self.inc_u8(&mut l);
                self.reg_hl.lo = l;
                4
            },
            0x2D => { 
                let mut l = self.reg_hl.lo;
                self.dec_u8(&mut l);
                self.reg_hl.lo = l;
                4
            },
            0x2E => { ld_u8_reg(self.get_byte(), &mut self.reg_hl.lo); 8 },
            0x2F => { 
                self.reg_af.hi = !self.reg_af.hi;
                self.update_subtract_flag(true);
                self.update_half_carry_flag(true);
                4
            },
            0x30 => {
                if !test_bit(self.reg_af.lo, 4) {
                    self.reg_pc = ((self.get_byte() as i8) as i32 + ((self.reg_pc as u32) as i32)) as u16;
                    12
                }
                else {
                    self.reg_pc += 1;
                    8
                }
            },
            0x31 => { ld_u16_reg_pair(self.get_word(), &mut self.reg_sp); 12 },
            0x32 => {
                self.memory_manager.borrow_mut().write_memory(self.reg_hl.get_pair(), self.reg_af.hi);
                dec_reg_pair(&mut self.reg_hl);
                8
            },
            0x33 => { inc_reg_pair(&mut self.reg_sp); 8 },
            0x34 => {
                let byte = &mut self.memory_manager.borrow_mut().read_memory(self.reg_hl.get_pair());
                self.inc_u8(byte);
                12
            },
            0x35 => {
                let byte = &mut self.memory_manager.borrow_mut().read_memory(self.reg_hl.get_pair());
                self.dec_u8(byte);
                12
            },
            0x36 => { 
                let byte = self.get_byte();
                self.memory_manager.borrow_mut().write_memory(self.reg_hl.get_pair(), byte);
                12
            },
            0x37 => {
                self.update_carry_flag(true);
                self.update_subtract_flag(false);
                self.update_half_carry_flag(false);
                4
            },
            0x38 => {
                if test_bit(self.reg_af.lo, 4) {
                    self.reg_pc = ((self.get_byte() as i8) as i32 + ((self.reg_pc as u32) as i32)) as u16;
                    12
                }
                else {
                    self.reg_pc += 1;
                    8
                }
            },
            0x39 => {
                let mut sp = self.reg_sp.get_pair();
                self.add_u16_hl(&mut sp);
                self.reg_sp.set_pair(sp);
                8
            },
            0x3A => {
                self.reg_af.hi = self.memory_manager.borrow_mut().read_memory(self.reg_hl.get_pair());
                dec_reg_pair(&mut self.reg_hl);
                8
            },
            0x3B => { 
                let val = self.reg_sp.get_pair().wrapping_sub(1);
                self.reg_sp.set_pair(val);
                8
            },
            0x3C => { 
                let mut a = self.reg_af.hi;
                self.inc_u8(&mut a);
                self.reg_af.hi = a;
                4
            },
            0x3D => { 
                let mut a = self.reg_af.hi;
                self.dec_u8(&mut a);
                self.reg_af.hi = a;
                4
            },
            0x3E => { ld_u8_reg(self.get_byte(), &mut self.reg_af.hi); 8 },
            0x3F => {
                let carry_set = test_bit(self.reg_af.lo, 4);
                self.update_carry_flag(!carry_set);
                self.update_subtract_flag(false);
                self.update_half_carry_flag(false);
                4
            },
            0x40 => { /* LD B, B */ 4 },
            0x41 => { ld_u8_reg(self.reg_bc.lo, &mut self.reg_bc.hi); 4 },
            0x42 => { ld_u8_reg(self.reg_de.hi, &mut self.reg_bc.hi); 4 },
            0x43 => { ld_u8_reg(self.reg_de.lo, &mut self.reg_bc.hi); 4 },
            0x44 => { ld_u8_reg(self.reg_hl.hi, &mut self.reg_bc.hi); 4 },
            0x45 => { ld_u8_reg(self.reg_de.lo, &mut self.reg_bc.hi); 4 },
            0x46 => { ld_u8_reg(self.memory_manager.borrow_mut().read_memory(self.reg_hl.get_pair()), &mut self.reg_bc.hi); 8 },
            0x47 => { ld_u8_reg(self.reg_af.hi, &mut self.reg_bc.hi); 4 },
            0x48 => { ld_u8_reg(self.reg_bc.hi, &mut self.reg_bc.lo); 4 },
            0x49 => { /* LD C, C */ 4 },
            0x4A => { ld_u8_reg(self.reg_de.hi, &mut self.reg_bc.lo); 4 },
            0x4B => { ld_u8_reg(self.reg_de.lo, &mut self.reg_bc.lo); 4 },
            0x4C => { ld_u8_reg(self.reg_hl.hi, &mut self.reg_bc.lo); 4 },
            0x4D => { ld_u8_reg(self.reg_de.lo, &mut self.reg_bc.lo); 4 },
            0x4E => { ld_u8_reg(self.memory_manager.borrow_mut().read_memory(self.reg_hl.get_pair()), &mut self.reg_bc.lo); 4 },
            0x4F => { ld_u8_reg(self.reg_af.hi, &mut self.reg_bc.lo); 4 },
            0x50 => { ld_u8_reg(self.reg_bc.hi, &mut self.reg_de.hi); 4 },
            0x51 => { ld_u8_reg(self.reg_bc.lo, &mut self.reg_de.hi); 4 },
            0x52 => { /* LD D, D */ 4 },
            0x53 => { ld_u8_reg(self.reg_de.lo, &mut self.reg_de.hi); 4 },
            0x54 => { ld_u8_reg(self.reg_hl.hi, &mut self.reg_de.hi); 4 },
            0x55 => { ld_u8_reg(self.reg_hl.lo, &mut self.reg_de.hi); 4 },
            0x56 => { ld_u8_reg(self.memory_manager.borrow_mut().read_memory(self.reg_hl.get_pair()), &mut self.reg_de.hi); 8 },
            0x57 => { ld_u8_reg(self.reg_af.hi, &mut self.reg_de.hi); 4 },
            0x58 => { ld_u8_reg(self.reg_bc.hi, &mut self.reg_de.lo); 4 },
            0x59 => { ld_u8_reg(self.reg_bc.lo, &mut self.reg_de.lo); 4 },
            0x5A => { ld_u8_reg(self.reg_de.hi, &mut self.reg_de.lo); 4 },
            0x5B => { /* LD E, E */ 4 },
            0x5C => { ld_u8_reg(self.reg_hl.hi, &mut self.reg_de.lo); 4 },
            0x5D => { ld_u8_reg(self.reg_hl.lo, &mut self.reg_de.lo); 4 },
            0x5E => { ld_u8_reg(self.memory_manager.borrow_mut().read_memory(self.reg_hl.get_pair()), &mut self.reg_de.lo); 8 },
            0x5F => { ld_u8_reg(self.reg_af.hi, &mut self.reg_de.lo); 4 },
            0x60 => { ld_u8_reg(self.reg_bc.hi, &mut self.reg_hl.hi); 4 },
            0x61 => { ld_u8_reg(self.reg_bc.lo, &mut self.reg_hl.hi); 4 },
            0x62 => { ld_u8_reg(self.reg_de.hi, &mut self.reg_hl.hi); 4 },
            0x63 => { ld_u8_reg(self.reg_de.lo, &mut self.reg_hl.hi); 4 },
            0x64 => { /* LD H, H */ 4 },
            0x65 => { ld_u8_reg(self.reg_hl.lo, &mut self.reg_hl.hi); 4 },
            0x66 => { ld_u8_reg(self.memory_manager.borrow_mut().read_memory(self.reg_hl.get_pair()), &mut self.reg_hl.hi); 8 },
            0x67 => { ld_u8_reg(self.reg_af.hi, &mut self.reg_hl.hi); 4 },
            0x68 => { ld_u8_reg(self.reg_bc.hi, &mut self.reg_hl.lo); 4 },
            0x69 => { ld_u8_reg(self.reg_bc.lo, &mut self.reg_hl.lo); 4 },
            0x6A => { ld_u8_reg(self.reg_de.hi, &mut self.reg_hl.lo); 4 },
            0x6B => { ld_u8_reg(self.reg_de.lo, &mut self.reg_hl.lo); 4 },
            0x6C => { ld_u8_reg(self.reg_hl.hi, &mut self.reg_hl.lo); 4 },
            0x6D => { /* LD L, L */ 4 },
            0x6E => { ld_u8_reg(self.memory_manager.borrow_mut().read_memory(self.reg_hl.get_pair()), &mut self.reg_hl.lo); 8 },
            0x6F => { ld_u8_reg(self.reg_af.hi, &mut self.reg_hl.lo); 4 },
            0x70 => { self.memory_manager.borrow_mut().write_memory(self.reg_hl.get_pair(), self.reg_bc.hi); 8 },
            0x71 => { self.memory_manager.borrow_mut().write_memory(self.reg_hl.get_pair(), self.reg_bc.lo); 8 },
            0x72 => { self.memory_manager.borrow_mut().write_memory(self.reg_hl.get_pair(), self.reg_de.hi); 8 },
            0x73 => { self.memory_manager.borrow_mut().write_memory(self.reg_hl.get_pair(), self.reg_de.lo); 8 },
            0x74 => { self.memory_manager.borrow_mut().write_memory(self.reg_hl.get_pair(), self.reg_hl.hi); 8 },
            0x75 => { self.memory_manager.borrow_mut().write_memory(self.reg_hl.get_pair(), self.reg_hl.lo); 8 },
            0x76 => { self.halted = true; 4 },
            0x77 => { self.memory_manager.borrow_mut().write_memory(self.reg_hl.get_pair(), self.reg_af.hi); 8 },
            0x78 => { ld_u8_reg(self.reg_bc.hi, &mut self.reg_af.hi); 4 },
            0x79 => { ld_u8_reg(self.reg_bc.lo, &mut self.reg_af.hi); 4 },
            0x7A => { ld_u8_reg(self.reg_de.hi, &mut self.reg_af.hi); 4 },
            0x7B => { ld_u8_reg(self.reg_de.lo, &mut self.reg_af.hi); 4 },
            0x7C => { ld_u8_reg(self.reg_hl.hi, &mut self.reg_af.hi); 4 },
            0x7D => { ld_u8_reg(self.reg_hl.lo, &mut self.reg_af.hi); 4 },
            0x7E => { ld_u8_reg(self.memory_manager.borrow_mut().read_memory(self.reg_hl.get_pair()), &mut self.reg_af.hi); 8 },
            0x7F => { /* LD A, A */ 4 },
            0x80 => {
                let val = self.reg_bc.hi;
                self.add_u8_a(val);
                4
            },
            0x81 => {
                let val = self.reg_bc.lo;
                self.add_u8_a(val);
                4
            },
            0x82 => {
                let val = self.reg_de.hi;
                self.add_u8_a(val);
                4
            },
            0x83 => {
                let val = self.reg_de.lo;
                self.add_u8_a(val);
                4
            },
            0x84 => {
                let val = self.reg_hl.hi;
                self.add_u8_a(val);
                4
            },
            0x85 => {
                let val = self.reg_hl.lo;
                self.add_u8_a(val);
                4
            },
            0x86 => {
                let val = self.memory_manager.borrow_mut().read_memory(self.reg_hl.get_pair());
                self.add_u8_a(val);
                8
            },
            0x87 => {
                let val = self.reg_af.hi;
                self.add_u8_a(val);
                4
            },
            0x88 => {
                let val = self.reg_bc.hi;
                self.adc_reg_a(val);
                4
            },
            0x89 => {
                let val = self.reg_bc.lo;
                self.adc_reg_a(val);
                4
            },
            0x8A => {
                let val = self.reg_de.hi;
                self.adc_reg_a(val);
                4
            },
            0x8B => {
                let val = self.reg_de.lo;
                self.adc_reg_a(val);
                4
            },
            0x8C => {
                let val = self.reg_hl.hi;
                self.adc_reg_a(val);
                4
            },
            0x8D => {
                let val = self.reg_hl.lo;
                self.adc_reg_a(val);
                4
            },
            0x8E => {
                let val = self.memory_manager.borrow_mut().read_memory(self.reg_hl.get_pair());
                self.adc_reg_a(val);
                8
            },
            0x8F => {
                let val = self.reg_af.hi;
                self.adc_reg_a(val);
                4
            },
            0x90 => {
                let val = self.reg_bc.hi;
                self.sub_u8_a(val);
                4
            },
            0x91 => {
                let val = self.reg_bc.lo;
                self.sub_u8_a(val);
                4
            },
            0x92 => {
                let val = self.reg_de.hi;
                self.sub_u8_a(val);
                4
            },
            0x93 => {
                let val = self.reg_de.lo;
                self.sub_u8_a(val);
                4
            },
            0x94 => {
                let val = self.reg_hl.hi;
                self.sub_u8_a(val);
                4
            },
            0x95 => {
                let val = self.reg_hl.lo;
                self.sub_u8_a(val);
                4
            },
            0x96 => {
                let val = self.memory_manager.borrow_mut().read_memory(self.reg_hl.get_pair());
                self.sub_u8_a(val);
                8
            },
            0x97 => {
                let val = self.reg_af.hi;
                self.sub_u8_a(val);
                4
            }
            0x98 => {
                let val = self.reg_bc.hi;
                self.sbc_reg_a(val);
                4
            }
            0x99 => {
                let val = self.reg_bc.lo;
                self.sbc_reg_a(val);
                4
            }
            0x9A => {
                let val = self.reg_de.hi;
                self.sbc_reg_a(val);
                4
            }
            0x9B => {
                let val = self.reg_de.lo;
                self.sbc_reg_a(val);
                4
            }
            0x9C => {
                let val = self.reg_hl.hi;
                self.sbc_reg_a(val);
                4
            }
            0x9D => {
                let val = self.reg_hl.lo;
                self.sbc_reg_a(val);
                4
            }
            0x9E => {
                let val = self.memory_manager.borrow_mut().read_memory(self.reg_hl.get_pair());
                self.sbc_reg_a(val);
                8
            },
            0x9F => {
                let val = self.reg_af.hi;
                self.sbc_reg_a(val);
                4
            }
            0xA0 => {
                let val = self.reg_bc.hi;
                self.and_reg_a(val);
                4
            },
            0xA1 => {
                let val = self.reg_bc.lo;
                self.and_reg_a(val);
                4
            },
            0xA2 => {
                let val = self.reg_de.hi;
                self.and_reg_a(val);
                4
            },
            0xA3 => {
                let val = self.reg_de.lo;
                self.and_reg_a(val);
                4
            },
            0xA4 => {
                let val = self.reg_hl.hi;
                self.and_reg_a(val);
                4
            },
            0xA5 => {
                let val = self.reg_hl.lo;
                self.and_reg_a(val);
                4
            },
            0xA6 => {
                let val = self.memory_manager.borrow_mut().read_memory(self.reg_hl.get_pair());
                self.and_reg_a(val);
                8
            },
            0xA7 => {
                let val = self.reg_af.hi;
                self.and_reg_a(val);
                4
            },
            0xA8 => {
                let val = self.reg_bc.hi;
                self.xor_reg_a(val);
                4
            },
            0xA9 => {
                let val = self.reg_bc.lo;
                self.xor_reg_a(val);
                4
            },
            0xAA => {
                let val = self.reg_de.hi;
                self.xor_reg_a(val);
                4
            },
            0xAB => {
                let val = self.reg_de.lo;
                self.xor_reg_a(val);
                4
            },
            0xAC => {
                let val = self.reg_hl.hi;
                self.xor_reg_a(val);
                4
            },
            0xAD => {
                let val = self.reg_hl.lo;
                self.xor_reg_a(val);
                4
            },
            0xAE => {
                let val = self.memory_manager.borrow_mut().read_memory(self.reg_hl.get_pair());
                self.xor_reg_a(val);
                8
            },
            0xAF => {
                let val = self.reg_af.hi;
                self.xor_reg_a(val);
                4
            },
            0xB0 => {
                let val = self.reg_bc.hi;
                self.or_reg_a(val);
                4
            },
            0xB1 => {
                let val = self.reg_bc.lo;
                self.or_reg_a(val);
                4
            },
            0xB2 => {
                let val = self.reg_de.hi;
                self.or_reg_a(val);
                4
            },
            0xB3 => {
                let val = self.reg_de.lo;
                self.or_reg_a(val);
                4
            },
            0xB4 => {
                let val = self.reg_hl.hi;
                self.or_reg_a(val);
                4
            },
            0xB5 => {
                let val = self.reg_hl.lo;
                self.or_reg_a(val);
                4
            },
            0xB6 => {
                let val = self.memory_manager.borrow_mut().read_memory(self.reg_hl.get_pair());
                self.or_reg_a(val);
                8
            },
            0xB7 => {
                let val = self.reg_af.hi;
                self.or_reg_a(val);
                4
            },
            0xB8 => {
                let val = self.reg_bc.hi;
                self.cp_reg_a(val);
                4
            },
            0xB9 => {
                let val = self.reg_bc.lo;
                self.cp_reg_a(val);
                4
            },
            0xBA => {
                let val = self.reg_de.hi;
                self.cp_reg_a(val);
                4
            },
            0xBB => {
                let val = self.reg_de.lo;
                self.cp_reg_a(val);
                4
            },
            0xBC => {
                let val = self.reg_hl.hi;
                self.cp_reg_a(val);
                4
            },
            0xBD => {
                let val = self.reg_hl.lo;
                self.cp_reg_a(val);
                4
            },
            0xBE => {
                let val = self.memory_manager.borrow_mut().read_memory(self.reg_hl.get_pair());
                self.cp_reg_a(val);
                8
            },
            0xBF => {
                let val = self.reg_af.hi;
                self.cp_reg_a(val);
                4
            },
            0xC0 => {
                if !test_bit(self.reg_af.lo, 7) {
                    self.reg_pc = self.stack_pop();
                    20
                }
                else {
                    8
                }
            },
            0xC1 => { 
                let val = self.stack_pop();
                self.reg_bc.set_pair(val);
                12
            },
            0xC2 => {
                if !test_bit(self.reg_af.lo, 7) {
                    self.reg_pc = self.get_word();
                    16
                }
                else {
                    self.reg_pc += 2;
                    12
                }
            },
            0xC3 => { self.reg_pc = self.get_word(); 16 },
            0xC4 => {
                if !test_bit(self.reg_af.lo, 7) {
                    let pc = self.reg_pc;
                    self.stack_push(pc + 2);
                    self.reg_pc = self.get_word();
                    24
                }
                else {
                    self.reg_pc += 2;
                    12
                }
            },
            0xC5 => { 
                let val = self.reg_bc.get_pair();
                self.stack_push(val);
                16
            },
            0xC6 => {
                let val = self.get_byte();
                self.add_u8_a(val);
                8
            },
            0xC7 => { self.call_routine(0x0000); 16 },
            0xC8 => {
                if test_bit(self.reg_af.lo, 7) {
                    self.reg_pc = self.stack_pop();
                    20
                }
                else {
                    8 
                }
            },
            0xC9 => { self.reg_pc = self.stack_pop(); 16 },
            0xCA => {
                if test_bit(self.reg_af.lo, 7) {
                    self.reg_pc = self.get_word();
                    16
                }
                else {
                    self.reg_pc += 2;
                    12
                }
            },
            0xCB => { self.extended_instruction() },
            0xCC => {
                if test_bit(self.reg_af.lo, 7) {
                    let pc = self.reg_pc;
                    self.stack_push(pc + 2);
                    self.reg_pc = self.get_word();
                    24
                }
                else {
                    self.reg_pc += 2;
                    12
                }
            },
            0xCD => {
                let pc = self.reg_pc;
                self.stack_push(pc + 2);
                self.reg_pc = self.get_word();
                24
            },
            0xCE => {
                let val = self.get_byte();
                self.adc_reg_a(val);
                8
            },
            0xCF => { self.call_routine(0x0008); 16 },
            0xD0 => {
                if !test_bit(self.reg_af.lo, 4) {
                    self.reg_pc = self.stack_pop();
                    20
                }
                else {
                    8
                }
            },
            0xD1 => {
                let val = self.stack_pop();
                self.reg_de.set_pair(val);
                12
            },
            0xD2 => {
                if !test_bit(self.reg_af.lo, 4) {
                    self.reg_pc = self.get_word();
                    16
                }
                else {
                    self.reg_pc += 2;
                    12
                }
            },
            0xD4 => {
                if !test_bit(self.reg_af.lo, 4) {
                    let pc = self.reg_pc;
                    self.stack_push(pc + 2);
                    self.reg_pc = self.get_word();
                    24
                }
                else {
                    self.reg_pc += 2;
                    12
                }
            },
            0xD5 => {
                let val = self.reg_de.get_pair();
                self.stack_push(val);
                16
            },
            0xD6 => {
                let val = self.get_byte();
                self.sub_u8_a(val);
                8
            }
            0xD7 => { self.call_routine(0x0010); 16 },
            0xD8 => {
                if test_bit(self.reg_af.lo, 4) {
                    self.reg_pc = self.stack_pop();
                    20
                }
                else {
                    8
                }
            },
            0xD9 => { 
                self.interrupts_enabled = true;
                self.reg_pc = self.stack_pop();
                16
            },
            0xDA => {
                if test_bit(self.reg_af.lo, 4) {
                    self.reg_pc = self.get_word();
                    16
                }
                else {
                    self.reg_pc += 2;
                    12
                }
            },
            0xDC => {
                if test_bit(self.reg_af.lo, 4) {
                    let pc = self.reg_pc;
                    self.stack_push(pc + 2);
                    self.reg_pc = self.get_word();
                    24
                }
                else {
                    self.reg_pc += 2;
                    12
                }
            },
            0xDE => {
                let val = self.get_byte();
                self.sbc_reg_a(val);
                8
            }
            0xDF => { self.call_routine(0x0018); 16 },
            0xE0 => { 
                let address = self.get_byte() as u16 | 0xFF00;
                self.memory_manager.borrow_mut().write_memory(address, self.reg_af.hi);
                12
            },
            0xE1 => {
                let val = self.stack_pop();
                self.reg_hl.set_pair(val);
                12
            },
            0xE2 => { 
                let address = self.reg_bc.lo as u16 | 0xFF00;
                self.memory_manager.borrow_mut().write_memory(address, self.reg_af.hi);
                8
            },
            0xE5 => {
                let val = self.reg_hl.get_pair();
                self.stack_push(val);
                16
            },
            0xE6 => {
                let val = self.get_byte();
                self.and_reg_a(val);
                8
            }
            0xE7 => { self.call_routine(0x0020); 16 },
            0xE8 => {
                let byte = self.get_byte() as i8 as i16 as u16;
                let sp = self.reg_sp.get_pair();
                self.reg_sp.set_pair(sp.wrapping_add(byte));
                self.update_half_carry_flag((byte & 0x000F) + (sp & 0x000F) > 0x000F);
                self.update_carry_flag((byte & 0x00FF) + (sp & 0x00FF) > 0x00FF);
                self.update_zero_flag(false);
                self.update_subtract_flag(false);
                16
            },
            0xE9 => { self.reg_pc = self.reg_hl.get_pair(); 4 },
            0xEA => { 
                let address = self.get_word();
                self.memory_manager.borrow_mut().write_memory(address, self.reg_af.hi);
                16
            },
            0xEE => {
                let val = self.get_byte();
                self.xor_reg_a(val);
                8
            }
            0xEF => { self.call_routine(0x0028); 16 },
            0xF0 => { 
                let address = self.get_byte() as u16 | 0xFF00;
                self.reg_af.hi = self.memory_manager.borrow_mut().read_memory(address);
                12
            },
            0xF1 => {
                let val = self.stack_pop() & 0xFFF0;
                self.reg_af.set_pair(val);
                12
            },
            0xF2 => { 
                let address = self.reg_bc.lo as u16 | 0xFF00;
                self.reg_af.hi = self.memory_manager.borrow_mut().read_memory(address);
                8
            },
            0xF3 => { self.interrupts_enabled = false; 4 },
            0xF5 => {
                let val = self.reg_af.get_pair();
                self.stack_push(val);
                16
            },
            0xF6 => {
                let val = self.get_byte();
                self.or_reg_a(val);
                8
            }
            0xF7 => { self.call_routine(0x0030); 16 },
            0xF8 => {
                let byte = self.get_byte() as i8 as i16 as u16;
                let sp = self.reg_sp.get_pair();
                self.reg_hl.set_pair(sp.wrapping_add(byte));
                self.update_half_carry_flag((byte & 0x000F) + (sp & 0x000F) > 0x000F);
                self.update_carry_flag((byte & 0x00FF) + (sp & 0x00FF) > 0x00FF);
                self.update_zero_flag(false);
                self.update_subtract_flag(false);
                12
            },
            0xF9 => { self.reg_sp.set_pair(self.reg_hl.get_pair()); 8 },
            0xFA => { 
                let address = self.get_word();
                self.reg_af.hi = self.memory_manager.borrow_mut().read_memory(address);
                16
            },
            0xFB => { self.interrupts_enabled = true; 4 },
            0xFE => {
                let val = self.get_byte();
                self.cp_reg_a(val);
                8
            }
            0xFF => { self.call_routine(0x0038); 16 },
            _ => panic!("Undefined opcode: 0x{:02X}", opcode)
        }
    }

    /// Executes an extended instruction.
    pub fn extended_instruction(&mut self) -> i32 {
        let opcode = self.get_byte();
        match opcode {
            0x00 => { 
                let mut b = self.reg_bc.hi;
                self.rlc_u8(&mut b);
                self.reg_bc.hi = b;
                8
            },
            0x01 => { 
                let mut c = self.reg_bc.lo;
                self.rlc_u8(&mut c);
                self.reg_bc.lo = c;
                8
            },
            0x02 => { 
                let mut d = self.reg_de.hi;
                self.rlc_u8(&mut d);
                self.reg_de.hi = d;
                8
            },
            0x03 => { 
                let mut e = self.reg_de.lo;
                self.rlc_u8(&mut e);
                self.reg_de.lo = e;
                8
            },
            0x04 => { 
                let mut h = self.reg_hl.hi;
                self.rlc_u8(&mut h);
                self.reg_hl.hi = h;
                8
            },
            0x05 => { 
                let mut l = self.reg_hl.lo;
                self.rlc_u8(&mut l);
                self.reg_hl.lo = l;
                8
            },
            0x06 => {
                let mut byte = self.memory_manager.borrow_mut().memory[self.reg_hl.get_pair() as usize];
                self.rlc_u8(&mut byte);
                self.memory_manager.borrow_mut().memory[self.reg_hl.get_pair() as usize] = byte;
                16
            },
            0x07 => { 
                let mut a = self.reg_af.hi;
                self.rlc_u8(&mut a);
                self.reg_af.hi = a;
                8
            },
            0x08 => { 
                let mut b = self.reg_bc.hi;
                self.rrc_u8(&mut b);
                self.reg_bc.hi = b;
                8
            },
            0x09 => { 
                let mut c = self.reg_bc.lo;
                self.rrc_u8(&mut c);
                self.reg_bc.lo = c;
                8
            },
            0x0A => { 
                let mut d = self.reg_de.hi;
                self.rrc_u8(&mut d);
                self.reg_de.hi = d;
                8
            },
            0x0B => { 
                let mut e = self.reg_de.lo;
                self.rrc_u8(&mut e);
                self.reg_de.lo = e;
                8
            },
            0x0C => { 
                let mut h = self.reg_hl.hi;
                self.rrc_u8(&mut h);
                self.reg_hl.hi = h;
                8
            },
            0x0D => { 
                let mut l = self.reg_hl.lo;
                self.rrc_u8(&mut l);
                self.reg_hl.lo = l;
                8
            },
            0x0E => {
                let mut byte = self.memory_manager.borrow_mut().memory[self.reg_hl.get_pair() as usize];
                self.rrc_u8(&mut byte);
                self.memory_manager.borrow_mut().memory[self.reg_hl.get_pair() as usize] = byte;
                16
            },
            0x0F => { 
                let mut a = self.reg_af.hi;
                self.rrc_u8(&mut a);
                self.reg_af.hi = a;
                8
            },
            0x10 => { 
                let mut b = self.reg_bc.hi;
                self.rl_u8(&mut b);
                self.reg_bc.hi = b;
                8
            },
            0x11 => { 
                let mut c = self.reg_bc.lo;
                self.rl_u8(&mut c);
                self.reg_bc.lo = c;
                8
            },
            0x12 => { 
                let mut d = self.reg_de.hi;
                self.rl_u8(&mut d);
                self.reg_de.hi = d;
                8
            },
            0x13 => { 
                let mut e = self.reg_de.lo;
                self.rl_u8(&mut e);
                self.reg_de.lo = e;
                8
            },
            0x14 => { 
                let mut h = self.reg_hl.hi;
                self.rl_u8(&mut h);
                self.reg_hl.hi = h;
                8
            },
            0x15 => { 
                let mut l = self.reg_hl.lo;
                self.rl_u8(&mut l);
                self.reg_hl.lo = l;
                8
            },
            0x16 => {
                let mut byte = self.memory_manager.borrow_mut().memory[self.reg_hl.get_pair() as usize];
                self.rl_u8(&mut byte);
                self.memory_manager.borrow_mut().memory[self.reg_hl.get_pair() as usize] = byte;
                16
            },
            0x17 => { 
                let mut a = self.reg_af.hi;
                self.rl_u8(&mut a);
                self.reg_af.hi = a;
                8
            },
            0x18 => { 
                let mut b = self.reg_bc.hi;
                self.rr_u8(&mut b);
                self.reg_bc.hi = b;
                8
            },
            0x19 => { 
                let mut c = self.reg_bc.lo;
                self.rr_u8(&mut c);
                self.reg_bc.lo = c;
                8
            },
            0x1A => { 
                let mut d = self.reg_de.hi;
                self.rr_u8(&mut d);
                self.reg_de.hi = d;
                8
            },
            0x1B => { 
                let mut e = self.reg_de.lo;
                self.rr_u8(&mut e);
                self.reg_de.lo = e;
                8
            },
            0x1C => { 
                let mut h = self.reg_hl.hi;
                self.rr_u8(&mut h);
                self.reg_hl.hi = h;
                8
            },
            0x1D => { 
                let mut l = self.reg_hl.lo;
                self.rr_u8(&mut l);
                self.reg_hl.lo = l;
                8
            },
            0x1E => {
                let mut byte = self.memory_manager.borrow_mut().memory[self.reg_hl.get_pair() as usize];
                self.rr_u8(&mut byte);
                self.memory_manager.borrow_mut().memory[self.reg_hl.get_pair() as usize] = byte;
                16
            },
            0x1F => { 
                let mut a = self.reg_af.hi;
                self.rr_u8(&mut a);
                self.reg_af.hi = a;
                8
            },
            0x20 => {
                let b = self.reg_bc.hi; 
                self.reg_bc.hi = self.sla_u8(b);
                8
            },
            0x21 => {
                let c = self.reg_bc.lo; 
                self.reg_bc.lo = self.sla_u8(c);
                8
            },
            0x22 => {
                let d = self.reg_de.hi; 
                self.reg_de.hi = self.sla_u8(d);
                8
            },
            0x23 => {
                let e = self.reg_de.lo; 
                self.reg_de.lo = self.sla_u8(e);
                8
            },
            0x24 => {
                let h = self.reg_hl.hi; 
                self.reg_hl.hi = self.sla_u8(h);
                8
            },
            0x25 => {
                let l = self.reg_hl.lo; 
                self.reg_hl.lo = self.sla_u8(l);
                8
            },
            0x26 => {
                let val = self.memory_manager.borrow_mut().memory[self.reg_hl.get_pair() as usize];
                self.memory_manager.borrow_mut().memory[self.reg_hl.get_pair() as usize] = self.sla_u8(val);
                16
            },
            0x27 => {
                let a = self.reg_af.hi; 
                self.reg_af.hi = self.sla_u8(a);
                8
            },
            0x28 => {
                let b = self.reg_bc.hi; 
                self.reg_bc.hi = self.sra_u8(b);
                8
            },
            0x29 => {
                let c = self.reg_bc.lo; 
                self.reg_bc.lo = self.sra_u8(c);
                8
            },
            0x2A => {
                let d = self.reg_de.hi; 
                self.reg_de.hi = self.sra_u8(d);
                8
            },
            0x2B => {
                let e = self.reg_de.lo; 
                self.reg_de.lo = self.sra_u8(e);
                8
            },
            0x2C => {
                let h = self.reg_hl.hi; 
                self.reg_hl.hi = self.sra_u8(h);
                8
            },
            0x2D => {
                let l = self.reg_hl.lo; 
                self.reg_hl.lo = self.sra_u8(l);
                8
            },
            0x2E => {
                let val = self.memory_manager.borrow_mut().memory[self.reg_hl.get_pair() as usize];
                self.memory_manager.borrow_mut().memory[self.reg_hl.get_pair() as usize] = self.sra_u8(val);
                16
            },
            0x2F => {
                let a = self.reg_af.hi; 
                self.reg_af.hi = self.sra_u8(a);
                8
            },
            0x30 => {
                swap_nybbles(&mut self.reg_bc.hi);
                let is_zero = self.reg_bc.hi == 0;
                self.update_zero_flag(is_zero);
                8
            },
            0x31 => {
                swap_nybbles(&mut self.reg_bc.lo);
                let is_zero = self.reg_bc.lo == 0;
                self.update_zero_flag(is_zero);
                8
            },
            0x32 => {
                swap_nybbles(&mut self.reg_de.hi);
                let is_zero = self.reg_de.hi == 0;
                self.update_zero_flag(is_zero);
                8
            },
            0x33 => {
                swap_nybbles(&mut self.reg_de.lo);
                let is_zero = self.reg_de.lo == 0;
                self.update_zero_flag(is_zero);
                8
            },
            0x34 => {
                swap_nybbles(&mut self.reg_hl.hi);
                let is_zero = self.reg_hl.hi == 0;
                self.update_zero_flag(is_zero);
                8
            },
            0x35 => {
                swap_nybbles(&mut self.reg_hl.lo);
                let is_zero = self.reg_hl.lo == 0;
                self.update_zero_flag(is_zero);
                8
            },
            0x36 => {
                swap_nybbles(&mut self.memory_manager.borrow_mut().memory[self.reg_hl.get_pair() as usize]);
                let is_zero = self.reg_bc.hi == 0;
                self.update_zero_flag(is_zero);
                16
            },
            0x37 => {
                swap_nybbles(&mut self.reg_af.hi);
                let is_zero = self.reg_af.hi == 0;
                self.update_zero_flag(is_zero);
                8
            },
            0x38 => {
                let b = self.reg_bc.hi; 
                self.reg_bc.hi = self.srl_u8(b);
                8
            },
            0x39 => {
                let c = self.reg_bc.lo; 
                self.reg_bc.lo = self.srl_u8(c);
                8
            },
            0x3A => {
                let d = self.reg_de.hi; 
                self.reg_de.hi = self.srl_u8(d);
                8
            },
            0x3B => {
                let e = self.reg_de.lo; 
                self.reg_de.lo = self.srl_u8(e);
                8
            },
            0x3C => {
                let h = self.reg_hl.hi; 
                self.reg_hl.hi = self.srl_u8(h);
                8
            },
            0x3D => {
                let l = self.reg_hl.lo; 
                self.reg_hl.lo = self.srl_u8(l);
                8
            },
            0x3E => {
                let val = self.memory_manager.borrow_mut().memory[self.reg_hl.get_pair() as usize];
                self.memory_manager.borrow_mut().memory[self.reg_hl.get_pair() as usize] = self.srl_u8(val);
                16
            },
            0x3F => {
                let a = self.reg_af.hi; 
                self.reg_af.hi = self.srl_u8(a);
                8
            },
            0x40 => {
                let reg = self.reg_bc.hi;
                self.update_zero_flag(test_bit(reg, 0));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x41 => {
                let reg = self.reg_bc.lo;
                self.update_zero_flag(test_bit(reg, 0));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x42 => {
                let reg = self.reg_de.hi;
                self.update_zero_flag(test_bit(reg, 0));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x43 => {
                let reg = self.reg_de.lo;
                self.update_zero_flag(test_bit(reg, 0));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x44 => {
                let reg = self.reg_hl.hi;
                self.update_zero_flag(test_bit(reg, 0));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x45 => {
                let reg = self.reg_hl.lo;
                self.update_zero_flag(test_bit(reg, 0));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x46 => {
                let val = self.memory_manager.borrow_mut().memory[self.reg_hl.get_pair() as usize];
                self.update_zero_flag(test_bit(val, 0));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                16
            },
            0x47 => {
                let reg = self.reg_af.hi;
                self.update_zero_flag(test_bit(reg, 0));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x48 => {
                let reg = self.reg_bc.hi;
                self.update_zero_flag(test_bit(reg, 1));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x49 => {
                let reg = self.reg_bc.lo;
                self.update_zero_flag(test_bit(reg, 1));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x4A => {
                let reg = self.reg_de.hi;
                self.update_zero_flag(test_bit(reg, 1));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x4B => {
                let reg = self.reg_de.lo;
                self.update_zero_flag(test_bit(reg, 1));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x4C => {
                let reg = self.reg_hl.hi;
                self.update_zero_flag(test_bit(reg, 1));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x4D => {
                let reg = self.reg_hl.lo;
                self.update_zero_flag(test_bit(reg, 1));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x4E => {
                let val = self.memory_manager.borrow_mut().memory[self.reg_hl.get_pair() as usize];
                self.update_zero_flag(test_bit(val, 1));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                16
            },
            0x4F => {
                let reg = self.reg_af.hi;
                self.update_zero_flag(test_bit(reg, 1));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x50 => {
                let reg = self.reg_bc.hi;
                self.update_zero_flag(test_bit(reg, 2));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x51 => {
                let reg = self.reg_bc.lo;
                self.update_zero_flag(test_bit(reg, 2));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x52 => {
                let reg = self.reg_de.hi;
                self.update_zero_flag(test_bit(reg, 2));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x53 => {
                let reg = self.reg_de.lo;
                self.update_zero_flag(test_bit(reg, 2));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x54 => {
                let reg = self.reg_hl.hi;
                self.update_zero_flag(test_bit(reg, 2));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x55 => {
                let reg = self.reg_hl.lo;
                self.update_zero_flag(test_bit(reg, 2));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x56 => {
                let val = self.memory_manager.borrow_mut().memory[self.reg_hl.get_pair() as usize];
                self.update_zero_flag(test_bit(val, 2));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                16
            },
            0x57 => {
                let reg = self.reg_af.hi;
                self.update_zero_flag(test_bit(reg, 2));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x58 => {
                let reg = self.reg_bc.hi;
                self.update_zero_flag(test_bit(reg, 3));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x59 => {
                let reg = self.reg_bc.lo;
                self.update_zero_flag(test_bit(reg, 3));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x5A => {
                let reg = self.reg_de.hi;
                self.update_zero_flag(test_bit(reg, 3));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x5B => {
                let reg = self.reg_de.lo;
                self.update_zero_flag(test_bit(reg, 3));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x5C => {
                let reg = self.reg_hl.hi;
                self.update_zero_flag(test_bit(reg, 3));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x5D => {
                let reg = self.reg_hl.lo;
                self.update_zero_flag(test_bit(reg, 3));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x5E => {
                let val = self.memory_manager.borrow_mut().memory[self.reg_hl.get_pair() as usize];
                self.update_zero_flag(test_bit(val, 3));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                16
            },
            0x5F => {
                let reg = self.reg_af.hi;
                self.update_zero_flag(test_bit(reg, 3));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x60 => {
                let reg = self.reg_bc.hi;
                self.update_zero_flag(test_bit(reg, 4));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x61 => {
                let reg = self.reg_bc.lo;
                self.update_zero_flag(test_bit(reg, 4));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x62 => {
                let reg = self.reg_de.hi;
                self.update_zero_flag(test_bit(reg, 4));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x63 => {
                let reg = self.reg_de.lo;
                self.update_zero_flag(test_bit(reg, 4));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x64 => {
                let reg = self.reg_hl.hi;
                self.update_zero_flag(test_bit(reg, 4));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x65 => {
                let reg = self.reg_hl.lo;
                self.update_zero_flag(test_bit(reg, 4));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x66 => {
                let val = self.memory_manager.borrow_mut().memory[self.reg_hl.get_pair() as usize];
                self.update_zero_flag(test_bit(val, 4));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                16
            },
            0x67 => {
                let reg = self.reg_af.hi;
                self.update_zero_flag(test_bit(reg, 4));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x68 => {
                let reg = self.reg_bc.hi;
                self.update_zero_flag(test_bit(reg, 5));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x69 => {
                let reg = self.reg_bc.lo;
                self.update_zero_flag(test_bit(reg, 5));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x6A => {
                let reg = self.reg_de.hi;
                self.update_zero_flag(test_bit(reg, 5));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x6B => {
                let reg = self.reg_de.lo;
                self.update_zero_flag(test_bit(reg, 5));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x6C => {
                let reg = self.reg_hl.hi;
                self.update_zero_flag(test_bit(reg, 5));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x6D => {
                let reg = self.reg_hl.lo;
                self.update_zero_flag(test_bit(reg, 5));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x6E => {
                let val = self.memory_manager.borrow_mut().memory[self.reg_hl.get_pair() as usize];
                self.update_zero_flag(test_bit(val, 5));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                16
            },
            0x6F => {
                let reg = self.reg_af.hi;
                self.update_zero_flag(test_bit(reg, 5));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x70 => {
                let reg = self.reg_bc.hi;
                self.update_zero_flag(test_bit(reg, 6));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x71 => {
                let reg = self.reg_bc.lo;
                self.update_zero_flag(test_bit(reg, 6));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x72 => {
                let reg = self.reg_de.hi;
                self.update_zero_flag(test_bit(reg, 6));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x73 => {
                let reg = self.reg_de.lo;
                self.update_zero_flag(test_bit(reg, 6));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x74 => {
                let reg = self.reg_hl.hi;
                self.update_zero_flag(test_bit(reg, 6));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x75 => {
                let reg = self.reg_hl.lo;
                self.update_zero_flag(test_bit(reg, 6));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x76 => {
                let val = self.memory_manager.borrow_mut().memory[self.reg_hl.get_pair() as usize];
                self.update_zero_flag(test_bit(val, 6));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                16
            },
            0x77 => {
                let reg = self.reg_af.hi;
                self.update_zero_flag(test_bit(reg, 6));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x78 => {
                let reg = self.reg_bc.hi;
                self.update_zero_flag(test_bit(reg, 7));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x79 => {
                let reg = self.reg_bc.lo;
                self.update_zero_flag(test_bit(reg, 7));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x7A => {
                let reg = self.reg_de.hi;
                self.update_zero_flag(test_bit(reg, 7));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x7B => {
                let reg = self.reg_de.lo;
                self.update_zero_flag(test_bit(reg, 7));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x7C => {
                let reg = self.reg_hl.hi;
                self.update_zero_flag(test_bit(reg, 7));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x7D => {
                let reg = self.reg_hl.lo;
                self.update_zero_flag(test_bit(reg, 7));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x7E => {
                let val = self.memory_manager.borrow_mut().memory[self.reg_hl.get_pair() as usize];
                self.update_zero_flag(test_bit(val, 7));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                16
            },
            0x7F => {
                let reg = self.reg_af.hi;
                self.update_zero_flag(test_bit(reg, 7));
                self.update_subtract_flag(false);
                self.update_half_carry_flag(true);
                8
            },
            0x80 => { reset_bit(&mut self.reg_bc.hi, 0); 8 },
            0x81 => { reset_bit(&mut self.reg_bc.lo, 0); 8 },
            0x82 => { reset_bit(&mut self.reg_de.hi, 0); 8 },
            0x83 => { reset_bit(&mut self.reg_de.lo, 0); 8 },
            0x84 => { reset_bit(&mut self.reg_hl.hi, 0); 8 },
            0x85 => { reset_bit(&mut self.reg_hl.lo, 0); 8 },
            0x86 => { reset_bit(&mut self.memory_manager.borrow_mut().memory[self.reg_hl.get_pair() as usize], 0); 16 },
            0x87 => { reset_bit(&mut self.reg_af.hi, 0); 8 },
            0x88 => { reset_bit(&mut self.reg_bc.hi, 1); 8 },
            0x89 => { reset_bit(&mut self.reg_bc.lo, 1); 8 },
            0x8A => { reset_bit(&mut self.reg_de.hi, 1); 8 },
            0x8B => { reset_bit(&mut self.reg_de.lo, 1); 8 },
            0x8C => { reset_bit(&mut self.reg_hl.hi, 1); 8 },
            0x8D => { reset_bit(&mut self.reg_hl.lo, 1); 8 },
            0x8E => { reset_bit(&mut self.memory_manager.borrow_mut().memory[self.reg_hl.get_pair() as usize], 1); 16 },
            0x8F => { reset_bit(&mut self.reg_af.hi, 1); 8 },
            0x90 => { reset_bit(&mut self.reg_bc.hi, 2); 8 },
            0x91 => { reset_bit(&mut self.reg_bc.lo, 2); 8 },
            0x92 => { reset_bit(&mut self.reg_de.hi, 2); 8 },
            0x93 => { reset_bit(&mut self.reg_de.lo, 2); 8 },
            0x94 => { reset_bit(&mut self.reg_hl.hi, 2); 8 },
            0x95 => { reset_bit(&mut self.reg_hl.lo, 2); 8 },
            0x96 => { reset_bit(&mut self.memory_manager.borrow_mut().memory[self.reg_hl.get_pair() as usize], 2); 16 },
            0x97 => { reset_bit(&mut self.reg_af.hi, 2); 8 },
            0x98 => { reset_bit(&mut self.reg_bc.hi, 3); 8 },
            0x99 => { reset_bit(&mut self.reg_bc.lo, 3); 8 },
            0x9A => { reset_bit(&mut self.reg_de.hi, 3); 8 },
            0x9B => { reset_bit(&mut self.reg_de.lo, 3); 8 },
            0x9C => { reset_bit(&mut self.reg_hl.hi, 3); 8 },
            0x9D => { reset_bit(&mut self.reg_hl.lo, 3); 8 },
            0x9E => { reset_bit(&mut self.memory_manager.borrow_mut().memory[self.reg_hl.get_pair() as usize], 3); 16 },
            0x9F => { reset_bit(&mut self.reg_af.hi, 3); 8 },
            0xA0 => { reset_bit(&mut self.reg_bc.hi, 4); 8 },
            0xA1 => { reset_bit(&mut self.reg_bc.lo, 4); 8 },
            0xA2 => { reset_bit(&mut self.reg_de.hi, 4); 8 },
            0xA3 => { reset_bit(&mut self.reg_de.lo, 4); 8 },
            0xA4 => { reset_bit(&mut self.reg_hl.hi, 4); 8 },
            0xA5 => { reset_bit(&mut self.reg_hl.lo, 4); 8 },
            0xA6 => { reset_bit(&mut self.memory_manager.borrow_mut().memory[self.reg_hl.get_pair() as usize], 4); 16 },
            0xA7 => { reset_bit(&mut self.reg_af.hi, 4); 8 },
            0xA8 => { reset_bit(&mut self.reg_bc.hi, 5); 8 },
            0xA9 => { reset_bit(&mut self.reg_bc.lo, 5); 8 },
            0xAA => { reset_bit(&mut self.reg_de.hi, 5); 8 },
            0xAB => { reset_bit(&mut self.reg_de.lo, 5); 8 },
            0xAC => { reset_bit(&mut self.reg_hl.hi, 5); 8 },
            0xAD => { reset_bit(&mut self.reg_hl.lo, 5); 8 },
            0xAE => { reset_bit(&mut self.memory_manager.borrow_mut().memory[self.reg_hl.get_pair() as usize], 5); 16 },
            0xAF => { reset_bit(&mut self.reg_af.hi, 5); 8 },
            0xB0 => { reset_bit(&mut self.reg_bc.hi, 6); 8 },
            0xB1 => { reset_bit(&mut self.reg_bc.lo, 6); 8 },
            0xB2 => { reset_bit(&mut self.reg_de.hi, 6); 8 },
            0xB3 => { reset_bit(&mut self.reg_de.lo, 6); 8 },
            0xB4 => { reset_bit(&mut self.reg_hl.hi, 6); 8 },
            0xB5 => { reset_bit(&mut self.reg_hl.lo, 6); 8 },
            0xB6 => { reset_bit(&mut self.memory_manager.borrow_mut().memory[self.reg_hl.get_pair() as usize], 6); 16 },
            0xB7 => { reset_bit(&mut self.reg_af.hi, 6); 8 },
            0xB8 => { reset_bit(&mut self.reg_bc.hi, 7); 8 },
            0xB9 => { reset_bit(&mut self.reg_bc.lo, 7); 8 },
            0xBA => { reset_bit(&mut self.reg_de.hi, 7); 8 },
            0xBB => { reset_bit(&mut self.reg_de.lo, 7); 8 },
            0xBC => { reset_bit(&mut self.reg_hl.hi, 7); 8 },
            0xBD => { reset_bit(&mut self.reg_hl.lo, 7); 8 },
            0xBE => { reset_bit(&mut self.memory_manager.borrow_mut().memory[self.reg_hl.get_pair() as usize], 7); 16 },
            0xBF => { reset_bit(&mut self.reg_af.hi, 7); 8 },
            0xC0 => { set_bit(&mut self.reg_bc.hi, 0); 8 },
            0xC1 => { set_bit(&mut self.reg_bc.lo, 0); 8 },
            0xC2 => { set_bit(&mut self.reg_de.hi, 0); 8 },
            0xC3 => { set_bit(&mut self.reg_de.lo, 0); 8 },
            0xC4 => { set_bit(&mut self.reg_hl.hi, 0); 8 },
            0xC5 => { set_bit(&mut self.reg_hl.lo, 0); 8 },
            0xC6 => { set_bit(&mut self.memory_manager.borrow_mut().memory[self.reg_hl.get_pair() as usize], 0); 16 },
            0xC7 => { set_bit(&mut self.reg_af.hi, 0); 8 },
            0xC8 => { set_bit(&mut self.reg_bc.hi, 1); 8 },
            0xC9 => { set_bit(&mut self.reg_bc.lo, 1); 8 },
            0xCA => { set_bit(&mut self.reg_de.hi, 1); 8 },
            0xCB => { set_bit(&mut self.reg_de.lo, 1); 8 },
            0xCC => { set_bit(&mut self.reg_hl.hi, 1); 8 },
            0xCD => { set_bit(&mut self.reg_hl.lo, 1); 8 },
            0xCE => { set_bit(&mut self.memory_manager.borrow_mut().memory[self.reg_hl.get_pair() as usize], 1); 16 },
            0xCF => { set_bit(&mut self.reg_af.hi, 1); 8 },
            0xD0 => { set_bit(&mut self.reg_bc.hi, 2); 8 },
            0xD1 => { set_bit(&mut self.reg_bc.lo, 2); 8 },
            0xD2 => { set_bit(&mut self.reg_de.hi, 2); 8 },
            0xD3 => { set_bit(&mut self.reg_de.lo, 2); 8 },
            0xD4 => { set_bit(&mut self.reg_hl.hi, 2); 8 },
            0xD5 => { set_bit(&mut self.reg_hl.lo, 2); 8 },
            0xD6 => { set_bit(&mut self.memory_manager.borrow_mut().memory[self.reg_hl.get_pair() as usize], 2); 16 },
            0xD7 => { set_bit(&mut self.reg_af.hi, 2); 8 },
            0xD8 => { set_bit(&mut self.reg_bc.hi, 3); 8 },
            0xD9 => { set_bit(&mut self.reg_bc.lo, 3); 8 },
            0xDA => { set_bit(&mut self.reg_de.hi, 3); 8 },
            0xDB => { set_bit(&mut self.reg_de.lo, 3); 8 },
            0xDC => { set_bit(&mut self.reg_hl.hi, 3); 8 },
            0xDD => { set_bit(&mut self.reg_hl.lo, 3); 8 },
            0xDE => { set_bit(&mut self.memory_manager.borrow_mut().memory[self.reg_hl.get_pair() as usize], 3); 16 },
            0xDF => { set_bit(&mut self.reg_af.hi, 3); 8 },
            0xE0 => { set_bit(&mut self.reg_bc.hi, 4); 8 },
            0xE1 => { set_bit(&mut self.reg_bc.lo, 4); 8 },
            0xE2 => { set_bit(&mut self.reg_de.hi, 4); 8 },
            0xE3 => { set_bit(&mut self.reg_de.lo, 4); 8 },
            0xE4 => { set_bit(&mut self.reg_hl.hi, 4); 8 },
            0xE5 => { set_bit(&mut self.reg_hl.lo, 4); 8 },
            0xE6 => { set_bit(&mut self.memory_manager.borrow_mut().memory[self.reg_hl.get_pair() as usize], 4); 16 },
            0xE7 => { set_bit(&mut self.reg_af.hi, 4); 8 },
            0xE8 => { set_bit(&mut self.reg_bc.hi, 5); 8 },
            0xE9 => { set_bit(&mut self.reg_bc.lo, 5); 8 },
            0xEA => { set_bit(&mut self.reg_de.hi, 5); 8 },
            0xEB => { set_bit(&mut self.reg_de.lo, 5); 8 },
            0xEC => { set_bit(&mut self.reg_hl.hi, 5); 8 },
            0xED => { set_bit(&mut self.reg_hl.lo, 5); 8 },
            0xEE => { set_bit(&mut self.memory_manager.borrow_mut().memory[self.reg_hl.get_pair() as usize], 5); 16 },
            0xEF => { set_bit(&mut self.reg_af.hi, 5); 8 },
            0xF0 => { set_bit(&mut self.reg_bc.hi, 6); 8 },
            0xF1 => { set_bit(&mut self.reg_bc.lo, 6); 8 },
            0xF2 => { set_bit(&mut self.reg_de.hi, 6); 8 },
            0xF3 => { set_bit(&mut self.reg_de.lo, 6); 8 },
            0xF4 => { set_bit(&mut self.reg_hl.hi, 6); 8 },
            0xF5 => { set_bit(&mut self.reg_hl.lo, 6); 8 },
            0xF6 => { set_bit(&mut self.memory_manager.borrow_mut().memory[self.reg_hl.get_pair() as usize], 6); 16 },
            0xF7 => { set_bit(&mut self.reg_af.hi, 6); 8 },
            0xF8 => { set_bit(&mut self.reg_bc.hi, 7); 8 },
            0xF9 => { set_bit(&mut self.reg_bc.lo, 7); 8 },
            0xFA => { set_bit(&mut self.reg_de.hi, 7); 8 },
            0xFB => { set_bit(&mut self.reg_de.lo, 7); 8 },
            0xFC => { set_bit(&mut self.reg_hl.hi, 7); 8 },
            0xFD => { set_bit(&mut self.reg_hl.lo, 7); 8 },
            0xFE => { set_bit(&mut self.memory_manager.borrow_mut().memory[self.reg_hl.get_pair() as usize], 7); 16 },
            0xFF => { set_bit(&mut self.reg_af.hi, 7); 8 },
            _ => panic!("Undefined extended opcode: 0x{:02X}", opcode)
        }
    }


    /// Adds src and register A together
    /// and stores the sum in A.
    pub fn add_u8_a(&mut self, src: u8) {
        let a = self.reg_af.hi;
        let sum = a.wrapping_add(src);
        self.reg_af.hi = sum;
        self.update_half_carry_flag((((src & 0x0F) + (a & 0x0F)) & 0x10) == 0x10);
        self.update_carry_flag(src as u16 + sum as u16 > 0xFF);
        self.update_zero_flag(sum == 0);
        self.update_subtract_flag(false);
    }

    /// Add function with carry bit.
    pub fn adc_reg_a(&mut self, src: u8) {
        let a = self.reg_af.hi;
        let carry = if test_bit(self.reg_af.lo, 4) { 1 } else { 0 };
        let sum = a.wrapping_add(src).wrapping_add(carry);
        self.reg_af.hi = sum;
        self.update_half_carry_flag((((src & 0x0F) + (a & 0x0F)) + carry & 0x10) == 0x10);
        self.update_carry_flag(src as u16 + sum as u16 > 0xFF);
        self.update_zero_flag(sum == 0);
        self.update_subtract_flag(false);
    }

    /// Adds a u16 into HL.
    pub fn add_u16_hl(&mut self, src: &mut u16) {
        let hl = self.reg_hl.get_pair();
        self.reg_hl.set_pair(hl.wrapping_add(*src));
        self.update_half_carry_flag((((*src & 0xFFF) + (hl & 0xFFF)) & 0x100) == 0x100);
        self.update_carry_flag(*src as u32 + hl as u32 > 0xFFFF);
        self.update_subtract_flag(false);
    }

    /// Increments an unsigned 8-bit value.
    pub fn inc_u8(&mut self, dest: &mut u8) {
        *dest = dest.wrapping_add(1);
        self.update_zero_flag(*dest == 0);
        self.update_half_carry_flag(((1 + (*dest & 0x0F)) & 0x10) == 0x10);
        self.update_subtract_flag(false);
    }

    /// Decrements an unsigned 8-bit value.
    pub fn dec_u8(&mut self, dest: &mut u8) {
        *dest = dest.wrapping_sub(1);
        self.update_zero_flag(*dest == 0);
        self.update_half_carry_flag((*dest & 0x0F) < 1);
        self.update_subtract_flag(true);
    }

    /// Subtracts src from register A
    /// and stores the sum in A.
    pub fn sub_u8_a(&mut self, src: u8) {
        let a = self.reg_af.hi;
        let sum = a.wrapping_sub(src);
        self.reg_af.hi = sum;
        self.update_half_carry_flag((a & 0x0F) < (sum & 0x0F));
        self.update_carry_flag((src as i32 - sum as i32) < 0);
        self.update_zero_flag(sum == 0);
        self.update_subtract_flag(true);
    }

    /// Subtract function with carry bit.
    pub fn sbc_reg_a(&mut self, src: u8) {
        let a = self.reg_af.hi;
        let carry = if test_bit(self.reg_af.lo, 4) { 1 } else { 0 };
        let sum = a.wrapping_sub(src).wrapping_sub(carry);
        self.reg_af.hi = sum;
        self.update_half_carry_flag((a & 0x0F) < (sum & 0x0F) + carry);
        self.update_carry_flag(src as u16 + sum as u16 > 0xFF);
        self.update_zero_flag(sum == 0);
        self.update_subtract_flag(false);
    }

    /// Performs a bitwise AND and saves
    /// the result in register A.
    pub fn and_reg_a(&mut self, src: u8) {
        let res = self.reg_af.hi & src;
        self.reg_af.hi = res;
        self.update_half_carry_flag(true);
        self.update_carry_flag(false);
        self.update_zero_flag(res == 0);
        self.update_subtract_flag(false);
    }

    /// Performs a bitwise OR and saves
    /// the result in register A.
    pub fn or_reg_a(&mut self, src: u8) {
        let res = self.reg_af.hi | src;
        self.reg_af.hi = res;
        self.update_half_carry_flag(false);
        self.update_carry_flag(false);
        self.update_zero_flag(res == 0);
        self.update_subtract_flag(false);
    }

    /// Performs a bitwise XOR and saves
    /// the result in register A.
    pub fn xor_reg_a(&mut self, src: u8) {
        let res = self.reg_af.hi ^ src;
        self.reg_af.hi = res;
        self.update_half_carry_flag(false);
        self.update_carry_flag(false);
        self.update_zero_flag(res == 0);
        self.update_subtract_flag(false);
    }

    /// Compares src to the value in
    /// register A.
    pub fn cp_reg_a(&mut self, src: u8) {
        let a = self.reg_af.hi;
        self.sub_u8_a(src);
        self.reg_af.hi = a;
    }

    /// Rotates a u8's bits left.
    pub fn rl_u8(&mut self, src: &mut u8) {
        let carry_occurred = *src >> 7 == 1;
        *src = *src << 1;
        if test_bit(self.reg_af.lo, 4) {
            *src |= 1;
        }
        self.update_half_carry_flag(false);
        self.update_carry_flag(carry_occurred);
        self.update_zero_flag(*src == 0);
        self.update_subtract_flag(false);
    }

    /// Rotates a u8's bits left with carry.
    pub fn rlc_u8(&mut self, src: &mut u8) {
        let carry_occurred = *src >> 7 == 1;
        *src = src.rotate_left(1);
        self.update_half_carry_flag(false);
        self.update_carry_flag(carry_occurred);
        self.update_zero_flag(*src == 0);
        self.update_subtract_flag(false);
    }

    /// Rotates a u8's bits right.
    pub fn rr_u8(&mut self, src: &mut u8) {
        let carry_occurred = *src & 1 == 1;
        *src = *src >> 1;
        if test_bit(self.reg_af.lo, 4) {
            *src |= 1 << 7;
        }
        self.update_half_carry_flag(false);
        self.update_carry_flag(carry_occurred);
        self.update_zero_flag(*src == 0);
        self.update_subtract_flag(false);
    }

    /// Rotates a u8's bits right with carry.
    pub fn rrc_u8(&mut self, src: &mut u8) {
        let carry_occurred = *src & 1 == 1;
        *src = src.rotate_right(1);
        self.update_half_carry_flag(false);
        self.update_carry_flag(carry_occurred);
        self.update_zero_flag(*src == 0);
        self.update_subtract_flag(false);
    }

    /// Performs an arithmetic shift left.
    pub fn sla_u8(&mut self, byte: u8) -> u8 {
        let new_byte = byte << 1;
        self.update_zero_flag(new_byte == 0);
        self.update_subtract_flag(false);
        self.update_half_carry_flag(false);
        self.update_carry_flag(((byte as u16) << 1) > 0xFF);
        new_byte
    }

    /// Performs an arithmetic shift right.
    pub fn sra_u8(&mut self, byte: u8) -> u8 {
        let new_byte = (byte >> 1) | (byte & 0x80);
        self.update_zero_flag(new_byte == 0);
        self.update_subtract_flag(false);
        self.update_half_carry_flag(false);
        self.update_carry_flag(false);
        new_byte
    }

    /// Performs an logical shift right.
    pub fn srl_u8(&mut self, byte: u8) -> u8 {
        let new_byte = byte >> 1;
        self.update_zero_flag(new_byte == 0);
        self.update_subtract_flag(false);
        self.update_half_carry_flag(false);
        self.update_carry_flag(byte & 1 == 1);
        new_byte
    }

    /// Adjusts register A to BCD.
    pub fn daa(&mut self) {
        let mut shift = 0;
        let flags = self.reg_af.lo;

        if flags & (1 << 5) != 0 {
            shift |= 0x6;
        }
        if flags & (1 << 4) != 0 {
            shift |= 0x60;
        }
        if flags & (1 << 6) == 0 {
            if self.reg_af.hi & 0xF > 9 {
                shift |= 0x6;
            }
            if self.reg_af.hi > 0x99 {
                shift |= 0x60;
            }
            self.reg_af.hi = self.reg_af.hi.wrapping_sub(shift);
        }
        else {
            self.reg_af.hi = self.reg_af.hi.wrapping_add(shift);
        }

        let is_zero = self.reg_af.hi == 0;
        self.update_zero_flag(is_zero);
        self.update_half_carry_flag(false);
        self.update_carry_flag(shift >= 0x60);
    }
}