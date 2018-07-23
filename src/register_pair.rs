pub struct RegisterPair {
    lo: u8,
    hi: u8,
}

impl RegisterPair {

    /// Overloaded constructor.
    pub fn new(pair: u16) -> RegisterPair {
        RegisterPair {
            lo: (pair & 0x00FF) as u8,
            hi: ((pair & 0xFF00) >> 8) as u8
        }
    }

    /// Getter for the low value.
    pub fn get_lo(&mut self) -> u8 {
        self.lo
    }

    /// Getter for the high value.
    pub fn get_hi(&mut self) -> u8 {
        self.hi
    }

    /// Setter for the low value.
    pub fn set_lo(&mut self, lo: u8) {
        self.lo = lo;
    }

    /// Setter for the high value.
    pub fn set_hi(&mut self, hi: u8) {
        self.hi = hi;
    }

    /// Setter for a register pair.
    pub fn set_pair(&mut self, pair: u16) {
        self.lo = (pair & 0x00FF) as u8;
        self.hi = ((pair & 0xFF00) >> 8) as u8;
    }

    /// Getter for a register pair.
    pub fn get_pair(&mut self) -> u16 {
        (self.hi as u16) << 8 | self.lo as u16
    }
}