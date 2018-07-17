use register::*;

pub struct RegisterPair {
    lo: Register,
    hi: Register,
}

impl RegisterPair {

    /// Overloaded constructor.
    pub fn new(pair: u16) -> RegisterPair {
        let lo = pair & 0x00FF;
        let hi = (pair & 0xFF00) >> 8;

        RegisterPair {
            lo: Register::new(lo as u8),
            hi: Register::new(hi as u8)
        }
    }

    /// Getter for the low value.
    pub fn get_lo(&mut self) -> &mut Register {
        &mut self.lo
    }

    /// Getter for the high value.
    pub fn get_hi(&mut self) -> &mut Register {
        &mut self.hi
    }

    /// Setter for a register pair.
    pub fn set_pair(&mut self, pair: u16) {
        self.lo.set_val((pair & 0x00FF) as u8);
        self.hi.set_val(((pair & 0xFF00) >> 8) as u8);
    }

    /// Getter for a register pair.
    pub fn get_pair(&mut self) -> u16 {
        (self.hi.get_val() as u16) << 8 | self.lo.get_val() as u16
    }
}