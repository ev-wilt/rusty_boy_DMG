mod core;
mod gameboy;
mod cartridge;
mod cpu;

use core::Core;

fn main() {
    let mut core = Core::new();
    println!("{:04X}", core.gameboy.cpu.reg_pair(0x01, 0x4D));
}