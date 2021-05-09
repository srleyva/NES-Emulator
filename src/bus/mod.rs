use std::fs::File;
use std::io::Read;

pub struct MemoryBus {
    memory: [u8; 0xFFFF],
}

impl MemoryBus {
    pub fn new(buffer: Vec<u8>) -> Self {
        let mut memory: [u8; 0xFFFF] = [0; 0xFFFF];
        memory[0x8000..(0x8000 + &buffer.len())].copy_from_slice(&buffer[..]);
        Self { memory }
    }

    pub fn from_rom(rom_path: &'static str) -> Self {
        let mut buffer: Vec<u8> = Vec::new();
        File::open(rom_path)
            .and_then(|mut file| file.read_to_end(&mut buffer))
            .expect("Could not read rom");
        Self::new(buffer)
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    pub fn write_byte(&mut self, addr: u16, byte: u8) {
        self.memory[addr as usize] = byte;
    }
}
