mod core;
mod gameboy;
mod cartridge;

use core::Core;

fn main() {
    let core = Core::new();
    println!("{}", core.gameboy.cartridge.rom.len());
    for i in 0..core.gameboy.cartridge.rom.len() {
        print!("{}", core.gameboy.cartridge.rom[i]);
    }
}