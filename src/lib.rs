pub mod gameboy;
pub mod cartridge;
pub mod cpu;
pub mod register_pair;
pub mod memory_manager;
pub mod display_manager;
pub mod interrupt_handler;
pub mod gamepad;
pub mod instructions;
#[cfg(test)]

pub mod tests {
    use memory_manager::MemoryManager;

    #[test]
    fn memory_manager_clock_enabled() {
        let mut memory_manager = MemoryManager::new();
        memory_manager.write_memory(0xFF07, 2);
        assert_eq!(memory_manager.clock_enabled(), true);
    }
}