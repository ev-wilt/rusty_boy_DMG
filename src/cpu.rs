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
        let mut word = (self.memory_manager.borrow_mut().read_memory(self.reg_sp.get_pair() + 1) as u16) << 8;
        word |= self.memory_manager.borrow_mut().read_memory(self.reg_sp.get_pair()) as u16;
        self.reg_sp.set_pair(prev + 2);
        word
    }

    /// Moves the PC and executes the next opcode,
    /// then returns the number of cycles it 
    /// took.
    pub fn interpret_opcode(&mut self) {

        // Don't run if halted
        if self.halted {
            // Return 4
        }

        let opcode = self.memory_manager.borrow_mut().read_memory(self.reg_pc);
        self.reg_pc += 1;
        println!("{:02X}", opcode);
        match opcode {
            0x00 => { /* NOP */ },
            0x01 => { ld_u16_reg_pair(self.get_word(), &mut self.reg_bc) },
            0x02 => { self.memory_manager.borrow_mut().write_memory(self.reg_bc.get_pair(), self.reg_af.get_hi()) },
            0x03 => { inc_reg_pair(&mut self.reg_bc) },
            0x04 => {},
            0x05 => {},
            0x06 => { ld_u8_reg(self.get_byte(), &mut self.reg_bc.get_hi()) },
            0x07 => {},
            0x08 => { 
                let address = self.get_word();
                self.memory_manager.borrow_mut().write_memory(address, self.reg_sp.get_lo());
                self.memory_manager.borrow_mut().write_memory(address + 1, self.reg_sp.get_hi());
            },
            0x09 => {},
            0x0A => { self.reg_af.set_hi(self.memory_manager.borrow_mut().read_memory(self.reg_bc.get_pair())) },
            0x0B => {},
            0x0C => {},
            0x0D => {},
            0x0E => { ld_u8_reg(self.get_byte(), &mut self.reg_bc.get_lo()) },
            0x0F => {},
            0x10 => { /* STOP */ },
            0x11 => { ld_u16_reg_pair(self.get_word(), &mut self.reg_de) },
            0x12 => { self.memory_manager.borrow_mut().write_memory(self.reg_de.get_pair(), self.reg_af.get_hi()) },
            0x13 => { inc_reg_pair(&mut self.reg_de) },
            0x14 => {},
            0x15 => {},
            0x16 => { ld_u8_reg(self.get_byte(), &mut self.reg_de.get_hi()) },
            0x17 => {},
            0x18 => { self.reg_pc = ((self.get_byte() as i8) as i32 + ((self.reg_pc as u32) as i32)) as u16 },
            0x19 => {},
            0x1A => { self.reg_af.set_hi(self.memory_manager.borrow_mut().read_memory(self.reg_de.get_pair())) },
            0x1B => {},
            0x1C => {},
            0x1D => {},
            0x1E => { ld_u8_reg(self.get_byte(), &mut self.reg_de.get_lo()) },
            0x1F => {},
            0x20 => {
                if !test_bit(self.reg_af.get_lo(), 7) {
                    self.reg_pc = ((self.get_byte() as i8) as i32 + ((self.reg_pc as u32) as i32)) as u16;
                }
            },
            0x21 => { ld_u16_reg_pair(self.get_word(), &mut self.reg_hl) },
            0x22 => {
                self.memory_manager.borrow_mut().write_memory(self.reg_hl.get_pair(), self.reg_af.get_hi());
                inc_reg_pair(&mut self.reg_hl);
            },
            0x23 => { inc_reg_pair(&mut self.reg_hl)},
            0x24 => {},
            0x25 => {},
            0x26 => { ld_u8_reg(self.get_byte(), &mut self.reg_hl.get_hi()) },
            0x27 => {},
            0x28 => {
                if test_bit(self.reg_af.get_lo(), 7) {
                    self.reg_pc = ((self.get_byte() as i8) as i32 + ((self.reg_pc as u32) as i32)) as u16;
                }
            },
            0x29 => {},
            0x2A => {
                self.reg_af.set_hi(self.memory_manager.borrow_mut().read_memory(self.reg_hl.get_pair()));
                inc_reg_pair(&mut self.reg_hl);
            },
            0x2B => {},
            0x2C => {},
            0x2D => {},
            0x2E => { ld_u8_reg(self.get_byte(), &mut self.reg_hl.get_lo()) },
            0x2F => {},
            0x30 => {
                if !test_bit(self.reg_af.get_lo(), 4) {
                    self.reg_pc = ((self.get_byte() as i8) as i32 + ((self.reg_pc as u32) as i32)) as u16;
                }
            },
            0x31 => { ld_u16_reg_pair(self.get_word(), &mut self.reg_sp) },
            0x32 => {
                self.memory_manager.borrow_mut().write_memory(self.reg_hl.get_pair(), self.reg_af.get_hi());
                dec_reg_pair(&mut self.reg_hl);
            },
            0x33 => { inc_reg_pair(&mut self.reg_sp) },
            0x34 => {},
            0x35 => {},
            0x36 => { 
                let byte = self.get_byte();
                self.memory_manager.borrow_mut().write_memory(self.reg_hl.get_pair(), byte);
            },
            0x37 => {},
            0x38 => {
                if test_bit(self.reg_af.get_lo(), 4) {
                    self.reg_pc = ((self.get_byte() as i8) as i32 + ((self.reg_pc as u32) as i32)) as u16;
                }
            },
            0x39 => {},
            0x3A => {
                self.reg_af.set_hi(self.memory_manager.borrow_mut().read_memory(self.reg_hl.get_pair()));
                dec_reg_pair(&mut self.reg_hl);
            },
            0x3B => {},
            0x3C => {},
            0x3D => {},
            0x3E => { ld_u8_reg(self.get_byte(), &mut self.reg_af.get_hi()) },
            0x3F => {},
            0x40 => { /* LD B, B */ },
            0x41 => { ld_u8_reg(self.reg_bc.get_lo(), &mut self.reg_bc.get_hi()) },
            0x42 => { ld_u8_reg(self.reg_de.get_hi(), &mut self.reg_bc.get_hi()) },
            0x43 => { ld_u8_reg(self.reg_de.get_lo(), &mut self.reg_bc.get_hi()) },
            0x44 => { ld_u8_reg(self.reg_hl.get_hi(), &mut self.reg_bc.get_hi()) },
            0x45 => { ld_u8_reg(self.reg_de.get_lo(), &mut self.reg_bc.get_hi()) },
            0x46 => { ld_u8_reg(self.memory_manager.borrow_mut().read_memory(self.reg_hl.get_pair()), &mut self.reg_bc.get_hi()) },
            0x47 => { ld_u8_reg(self.reg_af.get_hi(), &mut self.reg_bc.get_hi()) },
            0x48 => { ld_u8_reg(self.reg_bc.get_hi(), &mut self.reg_bc.get_lo()) },
            0x49 => { /* LD C, C */ },
            0x4A => { ld_u8_reg(self.reg_de.get_hi(), &mut self.reg_bc.get_lo()) },
            0x4B => { ld_u8_reg(self.reg_de.get_lo(), &mut self.reg_bc.get_lo()) },
            0x4C => { ld_u8_reg(self.reg_hl.get_hi(), &mut self.reg_bc.get_lo()) },
            0x4D => { ld_u8_reg(self.reg_de.get_lo(), &mut self.reg_bc.get_lo()) },
            0x4E => { ld_u8_reg(self.memory_manager.borrow_mut().read_memory(self.reg_hl.get_pair()), &mut self.reg_bc.get_lo()) },
            0x4F => { ld_u8_reg(self.reg_af.get_hi(), &mut self.reg_bc.get_lo()) },
            0x50 => { ld_u8_reg(self.reg_bc.get_hi(), &mut self.reg_de.get_hi()) },
            0x51 => { ld_u8_reg(self.reg_bc.get_lo(), &mut self.reg_de.get_hi()) },
            0x52 => { /* LD D, D */ },
            0x53 => { ld_u8_reg(self.reg_de.get_lo(), &mut self.reg_de.get_hi()) },
            0x54 => { ld_u8_reg(self.reg_hl.get_hi(), &mut self.reg_de.get_hi()) },
            0x55 => { ld_u8_reg(self.reg_hl.get_lo(), &mut self.reg_de.get_hi()) },
            0x56 => { ld_u8_reg(self.memory_manager.borrow_mut().read_memory(self.reg_hl.get_pair()), &mut self.reg_de.get_hi()) },
            0x57 => { ld_u8_reg(self.reg_af.get_hi(), &mut self.reg_de.get_hi()) },
            0x58 => { ld_u8_reg(self.reg_bc.get_hi(), &mut self.reg_de.get_lo()) },
            0x59 => { ld_u8_reg(self.reg_bc.get_lo(), &mut self.reg_de.get_lo()) },
            0x5A => { ld_u8_reg(self.reg_de.get_hi(), &mut self.reg_de.get_lo()) },
            0x5B => { /* LD E, E */ },
            0x5C => { ld_u8_reg(self.reg_hl.get_hi(), &mut self.reg_de.get_lo()) },
            0x5D => { ld_u8_reg(self.reg_hl.get_lo(), &mut self.reg_de.get_lo()) },
            0x5E => { ld_u8_reg(self.memory_manager.borrow_mut().read_memory(self.reg_hl.get_pair()), &mut self.reg_de.get_lo()) },
            0x5F => { ld_u8_reg(self.reg_af.get_hi(), &mut self.reg_de.get_lo()) },
            0x60 => { ld_u8_reg(self.reg_bc.get_hi(), &mut self.reg_hl.get_hi()) },
            0x61 => { ld_u8_reg(self.reg_bc.get_lo(), &mut self.reg_hl.get_hi()) },
            0x62 => { ld_u8_reg(self.reg_de.get_hi(), &mut self.reg_hl.get_hi()) },
            0x63 => { ld_u8_reg(self.reg_de.get_lo(), &mut self.reg_hl.get_hi()) },
            0x64 => { /* LD H, H */ },
            0x65 => { ld_u8_reg(self.reg_hl.get_lo(), &mut self.reg_hl.get_hi()) },
            0x66 => { ld_u8_reg(self.memory_manager.borrow_mut().read_memory(self.reg_hl.get_pair()), &mut self.reg_hl.get_hi()) },
            0x67 => { ld_u8_reg(self.reg_af.get_hi(), &mut self.reg_hl.get_hi()) },
            0x68 => { ld_u8_reg(self.reg_bc.get_hi(), &mut self.reg_hl.get_lo()) },
            0x69 => { ld_u8_reg(self.reg_bc.get_lo(), &mut self.reg_hl.get_lo()) },
            0x6A => { ld_u8_reg(self.reg_de.get_hi(), &mut self.reg_hl.get_lo()) },
            0x6B => { ld_u8_reg(self.reg_de.get_lo(), &mut self.reg_hl.get_lo()) },
            0x6C => { ld_u8_reg(self.reg_hl.get_hi(), &mut self.reg_hl.get_lo()) },
            0x6D => { /* LD L, L */ },
            0x6E => { ld_u8_reg(self.memory_manager.borrow_mut().read_memory(self.reg_hl.get_pair()), &mut self.reg_hl.get_lo()) },
            0x6F => { ld_u8_reg(self.reg_af.get_hi(), &mut self.reg_hl.get_lo()) },
            0x70 => { self.memory_manager.borrow_mut().write_memory(self.reg_hl.get_pair(), self.reg_bc.get_hi()) },
            0x71 => { self.memory_manager.borrow_mut().write_memory(self.reg_hl.get_pair(), self.reg_bc.get_lo()) },
            0x72 => { self.memory_manager.borrow_mut().write_memory(self.reg_hl.get_pair(), self.reg_de.get_hi()) },
            0x73 => { self.memory_manager.borrow_mut().write_memory(self.reg_hl.get_pair(), self.reg_de.get_lo()) },
            0x74 => { self.memory_manager.borrow_mut().write_memory(self.reg_hl.get_pair(), self.reg_hl.get_hi()) },
            0x75 => { self.memory_manager.borrow_mut().write_memory(self.reg_hl.get_pair(), self.reg_hl.get_lo()) },
            0x76 => { self.halted = true },
            0x77 => { self.memory_manager.borrow_mut().write_memory(self.reg_hl.get_pair(), self.reg_af.get_hi()) },
            0x78 => { ld_u8_reg(self.reg_bc.get_hi(), &mut self.reg_af.get_hi()) },
            0x79 => { ld_u8_reg(self.reg_bc.get_lo(), &mut self.reg_af.get_hi()) },
            0x7A => { ld_u8_reg(self.reg_de.get_hi(), &mut self.reg_af.get_hi()) },
            0x7B => { ld_u8_reg(self.reg_de.get_lo(), &mut self.reg_af.get_hi()) },
            0x7C => { ld_u8_reg(self.reg_hl.get_hi(), &mut self.reg_af.get_hi()) },
            0x7D => { ld_u8_reg(self.reg_hl.get_lo(), &mut self.reg_af.get_hi()) },
            0x7E => { ld_u8_reg(self.memory_manager.borrow_mut().read_memory(self.reg_hl.get_pair()), &mut self.reg_af.get_hi()) },
            0x7F => { /* LD A, A */ },
            0x80 => {},
            0x81 => {},
            0x82 => {},
            0x83 => {},
            0x84 => {},
            0x85 => {},
            0x86 => {},
            0x87 => {},
            0x88 => {},
            0x89 => {},
            0x8A => {},
            0x8B => {},
            0x8C => {},
            0x8D => {},
            0x8E => {},
            0x8F => {},
            0x90 => {},
            0x91 => {},
            0x92 => {},
            0x93 => {},
            0x94 => {},
            0x95 => {},
            0x96 => {},
            0x97 => {},
            0x98 => {},
            0x99 => {},
            0x9A => {},
            0x9B => {},
            0x9C => {},
            0x9D => {},
            0x9E => {},
            0x9F => {},
            0xA0 => {},
            0xA1 => {},
            0xA2 => {},
            0xA3 => {},
            0xA4 => {},
            0xA5 => {},
            0xA6 => {},
            0xA7 => {},
            0xA8 => {},
            0xA9 => {},
            0xAA => {},
            0xAB => {},
            0xAC => {},
            0xAD => {},
            0xAE => {},
            0xAF => {},
            0xB0 => {},
            0xB1 => {},
            0xB2 => {},
            0xB3 => {},
            0xB4 => {},
            0xB5 => {},
            0xB6 => {},
            0xB7 => {},
            0xB8 => {},
            0xB9 => {},
            0xBA => {},
            0xBB => {},
            0xBC => {},
            0xBD => {},
            0xBE => {},
            0xBF => {},
            0xC0 => {},
            0xC1 => { 
                let val = self.stack_pop();
                self.reg_bc.set_pair(val);
            },
            0xC2 => {
                if !test_bit(self.reg_af.get_lo(), 7) {
                    self.reg_pc = self.get_word();
                }
            },
            0xC3 => { self.reg_pc = self.get_word() },
            0xC4 => {},
            0xC5 => { 
                let val = self.reg_bc.get_pair();
                self.stack_push(val);
            },
            0xC6 => {},
            0xC7 => {},
            0xC8 => {},
            0xC9 => {},
            0xCA => {
                if test_bit(self.reg_af.get_lo(), 7) {
                    self.reg_pc = self.get_word();
                }
            },
            0xCB => {},
            0xCC => {},
            0xCD => {},
            0xCE => {},
            0xCF => {},
            0xD0 => {},
            0xD1 => {
                let val = self.stack_pop();
                self.reg_de.set_pair(val);
            },
            0xD2 => {
                if !test_bit(self.reg_af.get_lo(), 4) {
                    self.reg_pc = self.get_word();
                }
            },
            0xD4 => {},
            0xD5 => {
                let val = self.reg_de.get_pair();
                self.stack_push(val);
            },
            0xD6 => {},
            0xD7 => {},
            0xD8 => {},
            0xD9 => {},
            0xDA => {
                if test_bit(self.reg_af.get_lo(), 4) {
                    self.reg_pc = self.get_word();
                }
            },
            0xDC => {},
            0xDE => {},
            0xDF => {},
            0xE0 => {},
            0xE1 => {
                let val = self.stack_pop();
                self.reg_hl.set_pair(val);
            },
            0xE2 => {},
            0xE5 => {
                let val = self.reg_hl.get_pair();
                self.stack_push(val);
            },
            0xE6 => {},
            0xE7 => {},
            0xE8 => {},
            0xE9 => { self.reg_pc = self.reg_hl.get_pair() },
            0xEA => {},
            0xEE => {},
            0xEF => {},
            0xF0 => {},
            0xF1 => {
                let val = self.stack_pop();
                self.reg_af.set_pair(val);
            },
            0xF2 => {},
            0xF3 => { self.interrupts_enabled = false },
            0xF5 => {
                let val = self.reg_af.get_pair();
                self.stack_push(val);
            },
            0xF6 => {},
            0xF7 => {},
            0xF8 => {},
            0xF9 => {},
            0xFA => {},
            0xFB => { self.interrupts_enabled = true },
            0xFE => {},
            0xFF => {},
            _ => panic!("Undefined opcode: {:02X}", opcode)
        }
    }
}