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

    pub fn read_word(&self, address: u16) -> u16 {
        let least_sig_bits = self.read_byte(address) as u16;
        let most_sig_bits = self.read_byte(address + 1) as u16;
        (most_sig_bits << 8) | least_sig_bits
    }

    pub fn write_word(&mut self, address: u16, word: u16) {
        let least_sig_bits = (word & 0b00000000_11111111) as u8;
        let most_sig_bits = (word >> 8) as u8;
        self.write_byte(address, least_sig_bits);
        self.write_byte(address + 1, most_sig_bits);
    }
}

mod test {
    use super::*;

    #[test]
    fn test_write_read_word() {
        let mut memory_bus = MemoryBus::new(vec![]);
        memory_bus.write_word(0x8001, 0xFF);
        let word = memory_bus.read_word(0x8001);
        assert_eq!(word, 0xFF)
    }

    #[test]
    fn test_write_read_byte() {
        let mut memory_bus = MemoryBus::new(vec![]);
        memory_bus.write_byte(0x8001, 0x01);
        let word = memory_bus.read_byte(0x8001);
        assert_eq!(word, 0x01)
    }
}
