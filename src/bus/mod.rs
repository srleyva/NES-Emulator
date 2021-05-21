use super::rom::Rom;

pub struct MemoryBus {
    memory: [u8; 2048],
    rom: Rom,
}

const RAM: u16 = 0x0000;
const RAM_MIRRORS_END: u16 = 0x1FFF;
const PPU_REGISTERS: u16 = 0x2000;
const PPU_REGISTERS_MIRRORS_END: u16 = 0x3FFF;

impl MemoryBus {
    pub fn new(rom: Rom) -> Self {
        Self {
            memory: [0; 2048],
            rom,
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = address & 0b0000_0111_1111_1111;
                self.memory[mirror_down_addr as usize]
            }
            PPU_REGISTERS..=PPU_REGISTERS_MIRRORS_END => {
                let _mirror_down_addr = address & 0b0010_0000_0000_0111;
                todo!("PPU is not supported yet")
            }
            0x8000..=0xFFFF => self.read_from_rom(address),
            _ => {
                println!("Ignoring mem access at {}", address);
                self.memory[address as usize]
            }
        }
    }

    pub fn write_byte(&mut self, address: u16, byte: u8) {
        match address {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = address & 0b0000_0111_1111_1111;
                self.memory[mirror_down_addr as usize] = byte;
            }
            PPU_REGISTERS..=PPU_REGISTERS_MIRRORS_END => {
                let _mirror_down_addr = address & 0b0010_0000_0000_0111;
                todo!("PPU is not supported yet")
            }
            0x8000..=0xFFFF => panic!("Attempted to write to ROM Address space"),
            _ => {
                println!("Ignoring mem access at {}", address);
                self.memory[address as usize] = byte
            }
        }
    }

    pub fn read_word(&self, address: u16) -> u16 {
        let least_sig_bits = self.read_byte(address) as u16;
        let most_sig_bits = self.read_byte(address + 1) as u16;
        (most_sig_bits << 8) | least_sig_bits
    }

    pub fn write_word(&mut self, address: u16, word: u16) {
        let least_sig_bits = (word & 0b0000_0000_1111_1111) as u8;
        let most_sig_bits = (word >> 8) as u8;
        self.write_byte(address, least_sig_bits);
        self.write_byte(address + 1, most_sig_bits);
    }

    pub fn read_from_rom(&self, mut address: u16) -> u8 {
        address -= 0x8000;
        if self.rom.prg_rom.len() == 0x4000 && address >= 0x4000 {
            //mirror if needed
            address %= 0x4000;
        }
        self.rom.prg_rom[address as usize]
    }
}

#[cfg(test)]
mod test {
    use super::super::rom::Mirroring;
    use super::*;

    #[test]
    fn test_write_read_word() {
        let mut memory_bus = MemoryBus::new(Rom {
            prg_rom: vec![],
            chr_rom: vec![],
            mapper: 0,
            screen_mirroring: Mirroring::Horizontal,
        });
        memory_bus.write_word(0x800, 0xFF);
        let word = memory_bus.read_word(0x800);
        assert_eq!(word, 0xFF)
    }

    #[test]
    fn test_write_read_byte() {
        let mut memory_bus = MemoryBus::new(Rom {
            prg_rom: vec![],
            chr_rom: vec![],
            mapper: 0,
            screen_mirroring: Mirroring::Horizontal,
        });
        memory_bus.write_byte(0x800, 0x01);
        let word = memory_bus.read_byte(0x800);
        assert_eq!(word, 0x01)
    }
}
