mod core;
mod gameboy;
mod cartridge;
mod cpu;
mod register_pair;
mod memory_manager;
mod display_manager;
mod interrupt_handler;

use core::Core;

fn main() {
    let mut core = Core::new();
}