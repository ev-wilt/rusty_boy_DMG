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
    pub fn update_zero_flag(&mut self, result: u8) {
        if result == 0 {
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

    /// Adds src and register A together
    /// and stores the sum in A.
    pub fn add_u8_a(&mut self, src: u8) {
        let a = self.reg_af.hi;
        let sum = a.wrapping_add(src);
        self.reg_af.hi = sum;
        self.update_half_carry_flag((((src & 0x0F) + (a & 0x0F)) & 0x10) == 0x10);
        self.update_carry_flag(src as u16 + sum as u16 > 0xFF);
        self.update_zero_flag(sum);
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
        self.update_zero_flag(sum);
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
        self.update_zero_flag(*dest);
        self.update_half_carry_flag(((1 + (*dest & 0x0F)) & 0x10) == 0x10);
        self.update_subtract_flag(false);
    }

    /// Decrements an unsigned 8-bit value.
    pub fn dec_u8(&mut self, dest: &mut u8) {
        *dest = dest.wrapping_sub(1);
        self.update_zero_flag(*dest);
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
        self.update_zero_flag(sum);
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
        self.update_zero_flag(sum);
        self.update_subtract_flag(false);
    }

    /// Performs a bitwise AND and saves
    /// the result in register A.
    pub fn and_reg_a(&mut self, src: u8) {
        let res = self.reg_af.hi & src;
        self.reg_af.hi = res;
        self.update_half_carry_flag(true);
        self.update_carry_flag(false);
        self.update_zero_flag(res);
        self.update_subtract_flag(false);
    }

    /// Performs a bitwise OR and saves
    /// the result in register A.
    pub fn or_reg_a(&mut self, src: u8) {
        let res = self.reg_af.hi | src;
        self.reg_af.hi = res;
        self.update_half_carry_flag(false);
        self.update_carry_flag(false);
        self.update_zero_flag(res);
        self.update_subtract_flag(false);
    }

    /// Performs a bitwise XOR and saves
    /// the result in register A.
    pub fn xor_reg_a(&mut self, src: u8) {
        let res = self.reg_af.hi ^ src;
        self.reg_af.hi = res;
        self.update_half_carry_flag(false);
        self.update_carry_flag(false);
        self.update_zero_flag(res);
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
        self.update_zero_flag(*src);
        self.update_subtract_flag(false);
    }

    /// Rotates a u8's bits left with carry.
    pub fn rlc_u8(&mut self, src: &mut u8) {
        let carry_occurred = *src >> 7 == 1;
        *src = src.rotate_left(1);
        self.update_half_carry_flag(false);
        self.update_carry_flag(carry_occurred);
        self.update_zero_flag(*src);
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
        self.update_zero_flag(*src);
        self.update_subtract_flag(false);
    }

    /// Rotates a u8's bits right with carry.
    pub fn rrc_u8(&mut self, src: &mut u8) {
        let carry_occurred = *src & 1 == 1;
        *src = src.rotate_right(1);
        self.update_half_carry_flag(false);
        self.update_carry_flag(carry_occurred);
        self.update_zero_flag(*src);
        self.update_subtract_flag(false);
    }


    /// Executes an extended instruction.
    pub fn extended_instruction(&mut self) -> i32 {
        0
    }

    /// Moves the PC and executes the next opcode,
    /// then returns the number of cycles it 
    /// took.
    pub fn interpret_opcode(&mut self) -> i32 {

        // Don't run if halted
        if self.halted {
            return 4;
        }

        let opcode = self.memory_manager.borrow_mut().read_memory(self.reg_pc);
        self.reg_pc += 1;

        // println!("{:02X}", opcode);
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
            0x27 => { /* DAA */ 4 },
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
                let val = self.reg_sp.get_pair();
                self.reg_sp.set_pair(val - 1);
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
                self.update_zero_flag(1);
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
                self.update_zero_flag(1);
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
            _ => panic!("Undefined opcode: {:02X}", opcode)
        }
    }
}