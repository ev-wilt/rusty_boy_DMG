pub struct RegisterPair {
    pub lo: u8,
    pub hi: u8,
}

impl RegisterPair {

    /// Overloaded constructor.
    pub fn new(pair: u16) -> RegisterPair {
        RegisterPair {
            lo: (pair & 0x00FF) as u8,
            hi: ((pair & 0xFF00) >> 8) as u8
        }
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