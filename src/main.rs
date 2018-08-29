pub mod gameboy;
pub mod cartridge;
pub mod cpu;
pub mod register_pair;
pub mod memory_manager;
pub mod display_manager;
pub mod interrupt_handler;
pub mod gamepad;
pub mod instructions;

use gameboy::*;

use std::time::Duration;

fn main() {
    let mut gameboy = Gameboy::new();
    loop {
        if !gameboy.step() {
            break;
        }
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}