mod core;
mod gameboy;
mod cartridge;
mod cpu;
mod register_pair;
mod memory_manager;
mod display_manager;
mod interrupt_handler;
mod gamepad;

#[cfg(test)]

mod tests {
    use memory_manager::MemoryManager;

    #[test]
    fn memory_manager_clock_enabled() {
        let mut memory_manager = MemoryManager::new();
        memory_manager.write_memory(0xFF07, 2);
        assert_eq!(memory_manager.clock_enabled(), true);
    }
}