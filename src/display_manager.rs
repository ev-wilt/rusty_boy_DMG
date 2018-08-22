use memory_manager::*;
use interrupt_handler::*;
use std::rc::Rc;
use std::cell::RefCell;

use gameboy::sdl2::pixels::Color;
use gameboy::sdl2::rect::Rect;
use gameboy::sdl2::video::Window;
use gameboy::sdl2::render::Canvas;
use gameboy::sdl2::VideoSubsystem;

pub enum DisplayColor {
    White,
    LightGray,
    DarkGray,
    Black
}

pub struct DisplayManager {
    display: [[[u8; 3]; 144]; 160],
    remaining_cycles: i32,
    memory_manager: Rc<RefCell<MemoryManager>>,
    interrupt_handler: InterruptHandler,
    canvas: Canvas<Window>
}

impl DisplayManager {

    /// Default constructor.
    pub fn new(memory_manager: Rc<RefCell<MemoryManager>>, interrupt_handler: InterruptHandler, video_subsystem: &VideoSubsystem) -> DisplayManager {
        
        // Set up video
        let window = video_subsystem.window("Rusty Boy DMG", 160, 144)
            .opengl()
            .build()
            .unwrap();
        let mut canvas = window.into_canvas().build().unwrap();
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        DisplayManager {
            display: [[[0; 3]; 144]; 160],
            remaining_cycles: 456,
            memory_manager: memory_manager,
            interrupt_handler: interrupt_handler,
            canvas: canvas
        }
    }

    /// Draws the current contents of
    /// the display array to the canvas.
    pub fn draw_display(&mut self) {
        for i in 0..160 * 144 {
            let x = i % 160;
            let y = i / 160;

            let red = self.display[x][y][0];
            let green = self.display[x][y][1];
            let blue = self.display[x][y][2];
            self.canvas.set_draw_color(Color::RGB(red, green, blue));
            let _ = self.canvas.fill_rect(Rect::new(x as i32, y as i32, 1, 1));
        }

        self.canvas.present();
    }

    /// Returns the color of a pixel given its color ID
    /// and its address.
    pub fn get_color(&mut self, color_id: u8, address: u16) -> DisplayColor {
        let color_palette = self.memory_manager.borrow_mut().read_memory(address);
        let palette_hi: i32;
        let palette_lo: i32;

        match color_id {
            0 => { palette_hi = 1; palette_lo = 0},
            1 => { palette_hi = 3; palette_lo = 2},
            2 => { palette_hi = 5; palette_lo = 4},
            3 => { palette_hi = 7; palette_lo = 6},
            _ => { panic!("Invalid value for color ID: {}", color_id); }
        }

        let mut color_bits = if (color_palette & (1 << palette_hi)) >> palette_hi == 1 { 1 } else { 0 };
        color_bits <<= 1;
        color_bits |= if (color_palette & (1 << palette_lo)) >> palette_lo == 1 { 1 } else { 0 };
        let color: DisplayColor;

        match color_bits {
            0 => { color = DisplayColor::White },
            1 => { color = DisplayColor::LightGray },
            2 => { color = DisplayColor::DarkGray },
            3 => { color = DisplayColor::Black }
            _ => { panic!("Invalid value for color: {}", color_bits); }
        }
        color
    }

    /// Adds the current tiles in memory to
    /// the display.
    pub fn render_tiles(&mut self) {

        let bg_window_tile_data: u16;
        let tile_map_display: u16;
        let tile_y: u8;

        let scroll_y = self.memory_manager.borrow_mut().read_memory(0xFF42);
        let scroll_x = self.memory_manager.borrow_mut().read_memory(0xFF43);
        let window_y = self.memory_manager.borrow_mut().read_memory(0xFF4A);
        let window_x = self.memory_manager.borrow_mut().read_memory(0xFF4B).wrapping_sub(7);
        let mut window_enabled = false;
        let mut unsigned_data = true;

        if self.test_display_bit(5) && window_y <= self.memory_manager.borrow_mut().read_memory(0xFF44) {
            window_enabled = true;
        }

        if self.test_display_bit(4) {
            bg_window_tile_data = 0x8000;
        }
        else {
            bg_window_tile_data = 0x8800;
            unsigned_data = false;
        }

        if !window_enabled {
            if self.test_display_bit(3) {
                tile_map_display = 0x9C00;
            }
            else {
                tile_map_display = 0x9800;
            }
        }
        else {
            if self.test_display_bit(6) {
                tile_map_display = 0x9C00;
            }
            else {
                tile_map_display = 0x9800;
            }
        }

        if !window_enabled {
            tile_y = scroll_y.wrapping_add(self.memory_manager.borrow_mut().read_memory(0xFF44));
        }
        else {
            tile_y = self.memory_manager.borrow_mut().read_memory(0xFF44) - window_y;
        }

        let pixel_y = (tile_y / 8) as u16 * 32;

        // Draw all horizontal pixels for the 
        // current scanline
        for pixel in 0..160 {
            let mut tile_x = pixel + scroll_x;

            if window_enabled && pixel >= window_x {
                tile_x = pixel - window_x;
            }

            let pixel_x = (tile_x / 8) as u16;
            let tile_address = tile_map_display + pixel_x as u16 + pixel_y as u16;
            let tile_id: i16;

            if unsigned_data {
                tile_id = self.memory_manager.borrow_mut().read_memory(tile_address as u16) as i16;
            }
            else {
                tile_id = (self.memory_manager.borrow_mut().read_memory(tile_address as u16) as i8) as i16;
            }
            
            let mut tile_loc = bg_window_tile_data;
            if unsigned_data {
                tile_loc += tile_id as u16 * 16;
            }
            else {
                tile_loc += (tile_id as i8 as i16 + 128) as u16 * 16;
            }

            let current_line = (tile_y % 8) * 2;
            let line_data_lo = self.memory_manager.borrow_mut().read_memory(tile_loc + current_line as u16);
            let line_data_hi = self.memory_manager.borrow_mut().read_memory(tile_loc + current_line as u16 + 1);
            let color_loc = ((tile_x as i32 % 8) - 7) * -1;
            let mut color_id = if (line_data_hi & (1 << color_loc)) >> color_loc == 1 { 1 } else { 0 };
            color_id <<= 1;
            color_id |= if (line_data_lo & (1 << color_loc)) >> color_loc == 1 { 1 } else { 0 };
            let color = self.get_color(color_id, 0xFF47);
            let red: u8;
            let green: u8;
            let blue: u8;

            match color {
                DisplayColor::White => { red = 0xFF; green = 0xFF; blue = 0xFF },
                DisplayColor::LightGray => { red = 0xCC; green = 0xCC; blue = 0xCC },
                DisplayColor::DarkGray => { red = 0x77; green = 0x77; blue = 0x77 },
                DisplayColor::Black => { red = 0x00; green = 0x00; blue = 0x00 }
            }

            let current_scanline = self.memory_manager.borrow_mut().read_memory(0xFF44);
            if current_scanline > 143 || pixel > 159 {
                panic!("Setting color of pixel outside of visible display.
                        Scanline: {} should be in range 0-143, 
                        Pixel: {} should be in range 0-159", current_scanline, pixel);
            }
            self.display[pixel as usize][current_scanline as usize][0] = red;
            self.display[pixel as usize][current_scanline as usize][1] = green;
            self.display[pixel as usize][current_scanline as usize][2] = blue;
        }

    }

    /// Adds the current sprites in memeory
    /// to the display.
    pub fn render_sprites(&mut self) {

        for current_sprite in 0..40 {
            let sprite_x = self.memory_manager.borrow_mut().read_memory(0xFE00 + (current_sprite * 4)) as u16 as i32 - 16;
            let sprite_y = self.memory_manager.borrow_mut().read_memory(0xFE00 + (current_sprite * 4) + 1) as u16 as i32 - 8;
            let sprite_id = self.memory_manager.borrow_mut().read_memory(0xFE00 + (current_sprite * 4) + 2) as u16;
            let sprite_attrs = self.memory_manager.borrow_mut().read_memory(0xFE00 + (current_sprite * 4) + 3);
            let current_scanline = self.memory_manager.borrow_mut().read_memory(0xFF44) as i32;

            let flip_x = if (sprite_attrs & (1 << 5)) >> 5 == 1 { true } else { false }; 
            let flip_y = if (sprite_attrs & (1 << 6)) >> 6 == 1 { true } else { false }; 
            let sprite_size = if self.test_display_bit(2) { 16 } else { 8 };

            if current_scanline >= sprite_y && current_scanline < (sprite_y + sprite_size) {
                let sprite_line: u16;

                if flip_y {
                    sprite_line = (sprite_size - 1 - (current_scanline - sprite_y)) as u16;
                }
                else {
                    sprite_line = (current_scanline - sprite_y) as u16;
                }

                let data_address = 0x8000 + sprite_id * 16 + sprite_line * 2;
                let data_lo = self.memory_manager.borrow_mut().read_memory(data_address);
                let data_hi = self.memory_manager.borrow_mut().read_memory(data_address + 1);

                for sprite_pixel in 0..8 {
                    let mut color_loc = sprite_pixel;

                    if flip_x {
                        color_loc -= 7;
                        color_loc *= -1;
                    }

                    let mut color_id = if (data_hi & (1 << color_loc)) >> color_loc == 1 { 1 } else { 0 };
                    color_id <<= 1;
                    color_id |= if (data_lo & (1 << color_loc)) >> color_loc == 1 { 1 } else { 0 };
                    let color_address = if sprite_attrs & (1 << 4) >> 4 == 1 { 0xFF49 } else { 0xFF48 };
                    let mut color = self.get_color(color_id, color_address);
                    let red: u8;
                    let green: u8;
                    let blue: u8;

                    match color {
                        DisplayColor::White => { red = 0xFF; green = 0xFF; blue = 0xFF },
                        DisplayColor::LightGray => { red = 0xCC; green = 0xCC; blue = 0xCC },
                        DisplayColor::DarkGray => { red = 0x77; green = 0x77; blue = 0x77 },
                        DisplayColor::Black => { red = 0x00; green = 0x00; blue = 0x00 }
                    }

                    let pixel = sprite_x + (7 - sprite_pixel);
                    if current_scanline > 143 || current_scanline < 0 || pixel > 159 || pixel < 0 {
                        panic!("Setting color of pixel outside of visible display.
                            Scanline: {} should be in range 0-143, 
                            Pixel: {} should be in range 0-159", current_scanline, pixel);
                    }
                    self.display[pixel as usize][current_scanline as usize][0] = red;
                    self.display[pixel as usize][current_scanline as usize][1] = green;
                    self.display[pixel as usize][current_scanline as usize][2] = blue;
                }
            }
        }
    }

    /// Draws the current scanline.
    pub fn draw_scanline(&mut self) {
        let display_control = self.memory_manager.borrow_mut().read_memory(0xFF40);

        if display_control & 1 == 1 {
            self.render_tiles();
        }
        if (display_control & (1 << 1)) >> 1 == 1 {
            self.render_sprites();
        }
    }

    /// Determines whether the scanline
    /// needs to be updated.
    pub fn update_display(&mut self, cycles: i32) {
        self.set_display_status();
        
        // Update remaining cycles only if 
        // the display is enabled
        if self.test_display_bit(7) {
            self.remaining_cycles -= cycles;
        }
        else {
            return;
        }

        // Move to next scanline
        if self.remaining_cycles <= 0 {
            self.memory_manager.borrow_mut().memory[0xFF44] += 1;
            let next_scanline = self.memory_manager.borrow_mut().read_memory(0xFF44);
            self.remaining_cycles = 456;

            // V-Blank
            if next_scanline == 144 {
                self.interrupt_handler.request_interrupt(0);
            }

            // Reset scanline
            else if next_scanline > 153 {
                self.memory_manager.borrow_mut().memory[0xFF44] = 0;
            }

            else if next_scanline < 144 {
                self.draw_scanline();
            }
        }
    }

    /// Determines what the current status of display is and
    /// updates the current mode if necessary.
    pub fn set_display_status(&mut self) {
        let mut display_status = self.memory_manager.borrow_mut().read_memory(0xFF41);

        // Test if display is enabled
        if !self.test_display_bit(7) {
            self.remaining_cycles = 456;
            self.memory_manager.borrow_mut().memory[0xFF44] = 0;
            display_status &= 0xFC;
            display_status |= 1;
            self.memory_manager.borrow_mut().write_memory(0xFF41, display_status);
            return;
        }

        let previous_mode = display_status & 0x3;
        let new_mode: u8;
        let current_scanline = self.memory_manager.borrow_mut().read_memory(0xFF44);
        let mut request_interrupt = false;

        // V-Blank
        if current_scanline >= 144 {
            new_mode = 1;
            
            // Set bit 0 to 1, bit 1 to 0
            display_status |= 1;
            display_status &= 0xFD;
            if (display_status & (1 << 4)) >> 4 == 1 {
                request_interrupt = true;
            }
        }
        else {
            let mode_2_min = 456 - 80;
            let mode_3_min = mode_2_min - 172;

            // Mode 2
            if self.remaining_cycles >= mode_2_min {
                new_mode = 2;

                // Set bit 0 to 0, bit 1 to 1
                display_status |= 1 << 1;
                display_status &= 0xFE;
                if (display_status & (1 << 5)) >> 5 == 1 {
                    request_interrupt = true;
                }
            }

            // Mode 3
            else if self.remaining_cycles >= mode_3_min {
                new_mode = 3;
                
                // Set bit 0 to 1, bit 1 to 1
                display_status |= 3;
            }

            // Mode 0
            else {
                new_mode = 0;

                // Set bit 0 to 0, bit 1 to 0
                display_status &= 0xFC;
                if (display_status & (1 << 3)) >> 3 == 1 {
                    request_interrupt = true;
                }
            }
        }

        if request_interrupt && (new_mode != previous_mode) {
            self.interrupt_handler.request_interrupt(1);
        }

        if current_scanline == self.memory_manager.borrow_mut().read_memory(0xFF45) {
            display_status |= 1 << 2;
            if (display_status & (1 << 6) >> 6) == 1 {
                self.interrupt_handler.request_interrupt(1);
            }
        }
        else {
            // Set bit 2 to 0
            display_status &= 0xFB;
        }
        self.memory_manager.borrow_mut().write_memory(0xFF41, display_status);
    }

    /// Tests a bit at 0xFF40 to determine if the 
    /// whether the bit is on.
    pub fn test_display_bit(&mut self, bit: u8) -> bool {
        if (self.memory_manager.borrow_mut().read_memory(0xFF40) & (1 << bit)) >> bit == 1 { true } else { false }
    }
}