mod apu;
mod bus;
mod cpu;
mod gamepad;
mod ppu;

use bus::MemoryBus;
use cpu::CPU;

pub fn start_game(rom_path: &'static str) {
    let mut cpu = CPU::new(MemoryBus::from_rom(rom_path));
    cpu.start();
}
