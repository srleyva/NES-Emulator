mod address;
pub(crate) mod frame;
mod registers;
mod scroll;

use crate::rom::Mirroring;
use address::Address;
use registers::{Control, Mask, Status};
use scroll::Scroll;
use std::sync::mpsc;

use self::frame::Frame;

#[rustfmt::skip]

pub static SYSTEM_PALLETE: [(u8,u8,u8); 64] = [
   (0x80, 0x80, 0x80), (0x00, 0x3D, 0xA6), (0x00, 0x12, 0xB0), (0x44, 0x00, 0x96), (0xA1, 0x00, 0x5E),
   (0xC7, 0x00, 0x28), (0xBA, 0x06, 0x00), (0x8C, 0x17, 0x00), (0x5C, 0x2F, 0x00), (0x10, 0x45, 0x00),
   (0x05, 0x4A, 0x00), (0x00, 0x47, 0x2E), (0x00, 0x41, 0x66), (0x00, 0x00, 0x00), (0x05, 0x05, 0x05),
   (0x05, 0x05, 0x05), (0xC7, 0xC7, 0xC7), (0x00, 0x77, 0xFF), (0x21, 0x55, 0xFF), (0x82, 0x37, 0xFA),
   (0xEB, 0x2F, 0xB5), (0xFF, 0x29, 0x50), (0xFF, 0x22, 0x00), (0xD6, 0x32, 0x00), (0xC4, 0x62, 0x00),
   (0x35, 0x80, 0x00), (0x05, 0x8F, 0x00), (0x00, 0x8A, 0x55), (0x00, 0x99, 0xCC), (0x21, 0x21, 0x21),
   (0x09, 0x09, 0x09), (0x09, 0x09, 0x09), (0xFF, 0xFF, 0xFF), (0x0F, 0xD7, 0xFF), (0x69, 0xA2, 0xFF),
   (0xD4, 0x80, 0xFF), (0xFF, 0x45, 0xF3), (0xFF, 0x61, 0x8B), (0xFF, 0x88, 0x33), (0xFF, 0x9C, 0x12),
   (0xFA, 0xBC, 0x20), (0x9F, 0xE3, 0x0E), (0x2B, 0xF0, 0x35), (0x0C, 0xF0, 0xA4), (0x05, 0xFB, 0xFF),
   (0x5E, 0x5E, 0x5E), (0x0D, 0x0D, 0x0D), (0x0D, 0x0D, 0x0D), (0xFF, 0xFF, 0xFF), (0xA6, 0xFC, 0xFF),
   (0xB3, 0xEC, 0xFF), (0xDA, 0xAB, 0xEB), (0xFF, 0xA8, 0xF9), (0xFF, 0xAB, 0xB3), (0xFF, 0xD2, 0xB0),
   (0xFF, 0xEF, 0xA6), (0xFF, 0xF7, 0x9C), (0xD7, 0xE8, 0x95), (0xA6, 0xED, 0xAF), (0xA2, 0xF2, 0xDA),
   (0x99, 0xFF, 0xFC), (0xDD, 0xDD, 0xDD), (0x11, 0x11, 0x11), (0x11, 0x11, 0x11)
];

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

#[derive(Debug)]

pub(crate) struct PPU {
    pub(crate) chr_rom: Vec<u8>,
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

    scanline: u16,
    cycles: usize,

    nmi_sender: mpsc::Sender<bool>,
}

impl PPU {
    pub fn new(chr_rom: Vec<u8>, mirroring: Mirroring, nmi_sender: mpsc::Sender<bool>) -> Self {
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
            nmi_sender,
        }
    }

    pub fn render(&mut self, frame: &mut Frame) {
        let bank = self.ctrl.bknd_pattern_addr();

        for i in 0..0x03c0 {
            let tile = self.vram[i] as u16;
            let tile_x = i % 32;
            let tile_y = i / 32;
            let tile =
                &self.chr_rom[(bank + tile * 16) as usize..=(bank + tile * 16 + 15) as usize];

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
                    frame.set_pixel(tile_x * 8 + x, tile_y * 8 + y, rgb)
                }
            }
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
            PPUAddress::Controller => {
                let before_nmi_status = self.ctrl.generate_vblank_nmi();
                self.ctrl.update(data.into());
                if !before_nmi_status
                    && self.ctrl.generate_vblank_nmi()
                    && self.status.is_in_vblank()
                {
                    self.nmi_sender.send(true).unwrap();
                }
            }
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

    pub fn tick(&mut self, cycles: u8) -> bool {
        self.cycles += cycles as usize;
        if self.cycles >= 341 {
            self.cycles = self.cycles - 341;
            self.scanline += 1;

            if self.scanline == 241 {
                if self.ctrl.generate_vblank_nmi() {
                    self.status.set_vblank_status(true);
                    todo!("trigger interrupt")
                }
            }
        }
        return if self.scanline >= 262 {
            self.scanline = 0;
            self.status.reset_vblank_status();
            true
        } else {
            false
        };
    }
}
