mod core;

use std::env;
use core::Core;

fn main() {
    let mut core = Core::new();
    let args: Vec<String> = env::args().collect();
    let rom = core.load_rom(&args[1]);
    let rom = match rom {
        Ok(vector) => vector,
        Err(error) => panic!("{}", error)
    };
    for i in 0..rom.len() {
        print!("{}", rom[i]);
    }
}
