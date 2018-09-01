# Rusty Boy DMG

Rusty Boy DMG is an emulator for the original Nintendo Gameboy written in the Rust programming language. While support for the Gameboy Color may be added in the future, the current focus is on the original Gameboy model.

# Running a ROM

To run a ROM, you'll first need to build the emulator with the following command. You'll need SDL2 installed and cargo installed for this command to work.

```cargo build --release```

After that, the binary file will be created and placed in ```/target/release```.

Execute the binary and pass in the directory of the ROM you want to run.

```./rusty_boy_dmg /test_roms/rom_name.gb```

# References and Thanks

A big thank you to Imran Nazar and his fantastic article ["Gameboy Emulation in Javascript"](http://imrannazar.com/GameBoy-Emulation-in-JavaScript).

Another big thanks to the author of codeslinger.co.uk and [his great walkthrough on emulating the Gameboy](http://www.codeslinger.co.uk/pages/projects/gameboy.html).

Last but not least, a thanks to the contributors of the [rust-sdl2 library](https://github.com/Rust-SDL2/rust-sdl2).

Other references include:

* [Pan Docs](http://bgb.bircd.org/pandocs.htm)
* [Gameboy CPU Instruction Set](http://www.pastraiser.com/cpu/gameboy/gameboy_opcodes.html)
* [Gameboy Opcode Summary](http://www.devrs.com/gb/files/opcodes.html)
* The helpful people of the [/r/EmuDev subreddit](https://www.reddit.com/r/EmuDev)

