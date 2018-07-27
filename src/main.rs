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

fn main() {
    let mut gameboy = Gameboy::new();
    loop {
        if !gameboy.step() {
            break;
        }
    }
}