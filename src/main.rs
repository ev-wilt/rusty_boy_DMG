mod gameboy;
mod cartridge;
mod cpu;
mod register_pair;
mod memory_manager;
mod display_manager;
mod interrupt_handler;
mod gamepad;

use gameboy::*;

fn main() {
    let mut gameboy = Gameboy::new();
    loop {
        if !gameboy.step() {
            break;
        }
    }
}