use crate::cpu::interrupt::Interrupt;

use super::ppu::{PPUValue, PPU};
use super::rom::Rom;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemoryBus {
    memory: [u8; 2048],
    prg_rom: Vec<u8>,
    ppu: PPU,
    cycles: usize,
}

const RAM: u16 = 0x0000;
const RAM_MIRRORS_END: u16 = 0x1FFF;
const PPU_REGISTERS: u16 = 0x2000;
const PPU_REGISTERS_MIRRORS_END: u16 = 0x3FFF;
const OAM_DMA: u16 = 0x4014;

impl MemoryBus {
    pub fn new(rom: Rom) -> Self {
        let ppu = PPU::new(rom.chr_rom, rom.screen_mirroring);
        Self {
            memory: [0; 2048],
            prg_rom: rom.prg_rom,
            ppu,
            cycles: 0,
        }
    }

    pub fn poll_nmi_status(&self) -> Option<Interrupt> {
        self.ppu.nmi_interrupt.clone()
    }

    pub fn read_byte(&mut self, address: u16) -> u8 {
        match address {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = address & 0b0000_0111_1111_1111;
                self.memory[mirror_down_addr as usize]
            }
            PPU_REGISTERS..=PPU_REGISTERS_MIRRORS_END | OAM_DMA => {
                self.ppu.read_register(address).into()
            }
            0x8000..=0xFFFF => self.read_from_rom(address),
            _ => {
                println!("Ignoring mem access at {}", address);
                0
            }
        }
    }

    pub fn write_byte(&mut self, address: u16, data: u8) {
        match address {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = address & 0b11111111111;
                self.memory[mirror_down_addr as usize] = data;
            }
            PPU_REGISTERS..=PPU_REGISTERS_MIRRORS_END | OAM_DMA => {
                self.ppu.write_register(address, PPUValue::Byte(data))
            }
            0x8000..=0xFFFF => panic!("Attempt to write to Cartridge ROM space: {:x}", address),

            _ => {
                println!("Ignoring mem write-access at {}", address);
            }
        }
    }

    pub fn read_word(&mut self, address: u16) -> u16 {
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
        if self.prg_rom.len() == 0x4000 && address >= 0x4000 {
            //mirror if needed
            address %= 0x4000;
        }
        self.prg_rom[address as usize]
    }

    pub fn tick(&mut self, cycles: u8) {
        self.cycles += cycles as usize;
        self.ppu.tick(cycles * 3);
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
