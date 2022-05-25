mod address;
mod registers;
mod scroll;

use crate::rom::Mirroring;
use address::Address;
use registers::{Control, Mask, Status};
use scroll::Scroll;

#[derive(Debug)]
pub enum PPUAddress {
    CHRROM(u16),
    RAM(u16),
    PaletteTable(u16),

    // Registers
    Controller,
    Mask,
    Status,
    OAMAddress,
    OAMData,
    Scroll,
    Address,
    Data,
    OAMDMA,
}

#[derive(Debug)]
pub enum PPUValue {
    Byte(u8),
    Buffer(&'static [u8; 256]),
}

impl From<PPUValue> for u8 {
    fn from(data: PPUValue) -> Self {
        match data {
            PPUValue::Byte(value) => value,
            _ => panic!("cannot convert to byte"),
        }
    }
}

impl From<PPUValue> for &'static [u8; 256] {
    fn from(data: PPUValue) -> Self {
        match data {
            PPUValue::Buffer(value) => value,
            _ => panic!("cannot convert to buffer"),
        }
    }
}

impl From<u16> for PPUAddress {
    fn from(value: u16) -> Self {
        match value {
            0..=0x1fff => Self::CHRROM(value),
            0x2000 => Self::Controller,
            0x2001 => Self::Mask,
            0x2002 => Self::Status,
            0x2003 => Self::OAMAddress,
            0x2004 => Self::OAMData,
            0x2005 => Self::Scroll,
            0x2006 => Self::Address,
            0x2007 => Self::Data,
            0x2009..=0x2fff => Self::RAM(value),
            0x3000..=0x3eff => panic!(
                "addr space 0x3000..0x3eff is not expected to be used, requested = {} ",
                value
            ),
            0x3f00..=0x3fff => Self::PaletteTable(value),
            0x4014 => Self::OAMDMA,
            _ => panic!("Unsupported address {}", value),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]

pub(crate) struct PPU {
    chr_rom: Vec<u8>,
    palette_table: [u8; 32],
    vram: [u8; 2048],
    oam_addr: u8,
    oam_data: [u8; 256],
    mirroring: Mirroring,
    buffer: u8,

    address: Address,
    ctrl: Control,
    mask: Mask,
    status: Status,
    scroll: Scroll,
}

impl PPU {
    pub fn new(chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        Self {
            chr_rom,
            vram: [0; 2048],
            oam_data: [0; 64 * 4],
            oam_addr: 0,
            palette_table: [0; 32],
            mirroring,
            address: Address::default(),
            ctrl: Control::default(),
            status: Status::default(),
            mask: Mask::default(),
            scroll: Scroll::default(),
            buffer: 0,
        }
    }

    pub fn read<T>(&mut self, address: T) -> PPUValue
    where
        T: Into<PPUAddress>,
    {
        let address = address.into();
        self.address.increment(self.ctrl.vram_addr_increment());

        match address {
            PPUAddress::CHRROM(value) => {
                let result = self.buffer;
                self.buffer = self.chr_rom[value as usize];
                PPUValue::Byte(result)
            }
            PPUAddress::PaletteTable(value) => PPUValue::Byte(self.palette_table[value as usize]),
            PPUAddress::RAM(value) => {
                let result = self.buffer;
                self.buffer = self.vram[self.mirror_vram_addr(value) as usize];
                PPUValue::Byte(result)
            }
            PPUAddress::Status => {
                let data = self.status.snapshot();
                self.status.reset_vblank_status();
                self.address.reset_latch();
                self.scroll.reset_latch();
                PPUValue::Byte(data)
            }
            _ => panic!("Read to Write Only register: {:?}", address),
        }
    }

    pub fn write<T>(&mut self, address: T, data: PPUValue)
    where
        T: Into<PPUAddress>,
    {
        let address = address.into();
        match address {
            PPUAddress::Address => self.address.update(data.into()),
            PPUAddress::Controller => self.ctrl.update(data.into()),
            PPUAddress::Mask => self.mask.update(data.into()),
            PPUAddress::Scroll => self.scroll.write(data.into()),
            PPUAddress::OAMAddress => self.oam_addr = data.into(),
            PPUAddress::OAMData => {
                self.oam_data[self.oam_addr as usize] = data.into();
                self.oam_addr = self.oam_addr.wrapping_add(1)
            }
            PPUAddress::OAMDMA => {
                let data: &'static [u8; 256] = data.into();
                for x in data.iter() {
                    self.oam_data[self.oam_addr as usize] = *x;
                    self.oam_addr = self.oam_addr.wrapping_add(1);
                }
            }
            _ => panic!("Write on {:?} not supported", address),
        }
    }

    fn mirror_vram_addr(&self, addr: u16) -> u16 {
        let mirrored_vram = addr & 0b10111111111111; // mirror down 0x3000-0x3eff to 0x2000 - 0x2eff
        let vram_index = mirrored_vram - 0x2000; // to vram vector
        let name_table = vram_index / 0x400; // to the name table index
        match (&self.mirroring, name_table) {
            (Mirroring::Vertical, 2) | (Mirroring::Vertical, 3) => vram_index - 0x800,
            (Mirroring::Horizontal, 2) => vram_index - 0x400,
            (Mirroring::Horizontal, 1) => vram_index - 0x400,
            (Mirroring::Horizontal, 3) => vram_index - 0x800,
            _ => vram_index,
        }
    }
}
