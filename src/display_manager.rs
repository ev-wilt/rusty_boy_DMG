pub struct DisplayManager {
    display: [[[u8; 3]; 144]; 160]
}

impl DisplayManager {

    /// Default constructor.
    pub fn new() -> DisplayManager {
        DisplayManager {
            display: [[[0; 3]; 144]; 160]
        }
    }

    pub fn printcrap(&mut self) {
        self.display[160/2][144/2][4] = 0;
    }
}