pub struct Register {
    val: u8
}

impl Register {

    /// Overloaded constructor.
    pub fn new(val: u8) -> Register {
        Register {
            val: val
        }
    }

    /// Setter for the value.
    pub fn set_val(&mut self, val: u8) {
        self.val = val;
    }

    /// Getter for a value.
    pub fn get_val(&mut self) -> u8{
        self.val
    }
}