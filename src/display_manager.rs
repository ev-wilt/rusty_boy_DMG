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

    pub fn update_display(&mut self) {

    }

    pub fn set_display_status(&mut self) {

    }

    pub fn display_enabled(&mut self) {

    }
}