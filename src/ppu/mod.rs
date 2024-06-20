mod address;
mod registers;
pub mod render;
mod scroll;

use crate::{cpu::interrupt::InterruptType, ppu::render::SYSTEM_PALLETE, rom::Mirroring};
use address::Address;
use registers::{Control, Mask, Status};
use scroll::Scroll;

use self::render::Frame;

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

impl Into<PPUValue> for u8 {
    fn into(self) -> PPUValue {
        PPUValue::Byte(self)
    }
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
            0x2000 => Self::Controller, // write-only
            0x2001 => Self::Mask,       // write-only
            0x2002 => Self::Status,
            0x2003 => Self::OAMAddress, // write-only
            0x2004 => Self::OAMData,
            0x2005 => Self::Scroll,  // write-only
            0x2006 => Self::Address, // write-only
            0x2007 => Self::Data,
            0x2009..=0x2fff => Self::RAM(value),
            0x3000..=0x3eff => panic!(
                "addr space 0x3000..0x3eff is not expected to be used, requested = {} ",
                value
            ),
            0x3f00..=0x3fff => Self::PaletteTable(value),
            0x4014 => Self::OAMDMA, // write-only
            _ => panic!("Unsupported address {}", value),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]

pub struct PPU {
    // PPU MemoryMap
    // 0x4000 - 0x3f00
    palette_table: [u8; 32],
    // 0x3f00 - 0x2000
    vram: [u8; 2048],
    // 0x2000 - 0x0000
    chr_rom: Vec<u8>,

    oam_addr: u8,
    oam_data: [u8; 256],
    mirroring: Mirroring,
    buffer: u8,
    pub(crate) nmi_interrupt: Option<InterruptType>,

    // PPU Registers
    ctrl: Control,
    mask: Mask,
    status: Status,
    scroll: Scroll,
    address: Address,

    // screen
    scanline: u16,
    cycles: usize,
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
            scanline: 0,
            cycles: 0,
            nmi_interrupt: None,
        }
    }

    pub fn read_register<T>(&mut self, register: T) -> PPUValue
    where
        T: Into<PPUAddress>,
    {
        let register = register.into();
        match register {
            PPUAddress::Controller
            | PPUAddress::Mask
            | PPUAddress::OAMAddress
            | PPUAddress::Scroll
            | PPUAddress::Address
            | PPUAddress::OAMDMA => {
                panic!("{:?} is a write only register", register)
            }
            PPUAddress::OAMData => self.oam_data[self.oam_addr as usize].into(),
            PPUAddress::Data => self.read_data(),
            PPUAddress::Status => self.status.bits().into(),
            _ => panic!("register not provided: {:?}", register),
        }
    }

    pub fn show_tile(&self, bank: usize, tile_n: usize) -> Frame {
        assert!(bank <= 1);

        let mut frame = Frame::default();
        let bank = (bank * 0x1000) as usize;

        let tile = &self.chr_rom[(bank + tile_n * 16)..=(bank + tile_n * 16 + 15)];

        for y in 0..=7 {
            let mut upper = tile[y];
            let mut lower = tile[y + 8];

            for x in (0..=7).rev() {
                let value = (1 & upper) << 1 | (1 & lower);
                upper = upper >> 1;
                lower = lower >> 1;
                let rgb = match value {
                    0 => SYSTEM_PALLETE[0x01],
                    1 => SYSTEM_PALLETE[0x23],
                    2 => SYSTEM_PALLETE[0x27],
                    3 => SYSTEM_PALLETE[0x30],
                    _ => panic!("can't be"),
                };
                frame.set_pixel(x, y, rgb)
            }
        }

        frame
    }

    pub fn write_register<T>(&mut self, register: T, data: PPUValue)
    where
        T: Into<PPUAddress>,
    {
        let register = register.into();
        match register {
            PPUAddress::Controller => {
                let before_nmi_status = self.ctrl.generate_vblank_nmi();
                self.ctrl.update(data.into());
                if !before_nmi_status
                    && self.ctrl.generate_vblank_nmi()
                    && self.status.is_in_vblank()
                {
                    self.nmi_interrupt = Some(InterruptType::NMI);
                }
            }
            PPUAddress::Mask => self.mask.update(data.into()),
            PPUAddress::Status => panic!("status is a r/o register but was written to!"),
            PPUAddress::OAMAddress => self.oam_addr = data.into(),
            PPUAddress::OAMData => {
                self.oam_data[self.oam_addr as usize] = data.into();
                self.oam_addr = self.oam_addr.wrapping_add(1)
            }
            PPUAddress::Scroll => self.scroll.write(data.into()),
            PPUAddress::Address => self.address.update(data.into()),
            PPUAddress::Data => self.write_data(data),
            PPUAddress::OAMDMA => {
                let data: &'static [u8; 256] = data.into();
                for x in data.iter() {
                    self.oam_data[self.oam_addr as usize] = *x;
                    self.oam_addr = self.oam_addr.wrapping_add(1);
                }
            }
            _ => panic!("register not provided: {:?}", register),
        }
    }

    fn read_data(&mut self) -> PPUValue {
        let address = self.address.get().into();
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

    fn write_data(&mut self, data: PPUValue) {
        let address: PPUAddress = self.address.get().into();
        self.address.increment(self.ctrl.vram_addr_increment());

        match address {
            PPUAddress::RAM(addr) => {
                self.vram[self.mirror_vram_addr(addr) as usize] = data.into();
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

    pub fn tick(&mut self, cycles: u8) -> bool {
        self.cycles += cycles as usize;
        if self.cycles >= 341 {
            self.cycles = self.cycles - 341;
            self.scanline += 1;

            if self.scanline == 241 {
                if self.ctrl.generate_vblank_nmi() {
                    self.status.set_vblank_status(true);
                    todo!("Should trigger NMI interrupt")
                }
            }

            if self.scanline >= 262 {
                self.scanline = 0;
                self.status.reset_vblank_status();
                return true;
            }
        }
        return false;
    }
}
