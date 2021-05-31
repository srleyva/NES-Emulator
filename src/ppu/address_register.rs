#[derive(Default)]
pub struct AddrRegister {
    high_byte: u8,
    low_byte: u8,
    hi_ptr: bool,
}

impl AddrRegister {
    fn set(&mut self, data: u16) {
        self.high_byte = (data >> 8) as u8;
        self.low_byte = (data & 0xff) as u8;
    }

    pub fn update(&mut self, data: u8) {
        if self.hi_ptr {
            self.high_byte = data;
        } else {
            self.low_byte = data;
        }

        if self.get() > 0x3fff {
            //mirror down addr above 0x3fff
            self.set(self.get() & 0b11111111111111);
        }
        self.hi_ptr = !self.hi_ptr;
    }

    pub fn increment(&mut self, inc: u8) {
        let lo = self.low_byte;
        self.low_byte = self.low_byte.wrapping_add(inc);
        if lo > self.low_byte {
            self.high_byte = self.high_byte.wrapping_add(1);
        }
        if self.get() > 0x3fff {
            self.set(self.get() & 0b11111111111111); //mirror down addr above 0x3fff
        }
    }

    pub fn reset_latch(&mut self) {
        self.hi_ptr = true;
    }

    pub fn get(&self) -> u16 {
        ((self.high_byte as u16) << 8) | (self.low_byte as u16)
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_addr_register_set() {
        let mut addr_register = AddrRegister::default();
        addr_register.set(12);

        assert_eq!(addr_register.get(), 12);
    }

    #[test]
    fn test_addr_register_incr() {
        let mut addr_register = AddrRegister::default();
        addr_register.set(12);
        addr_register.increment(12);

        assert_eq!(addr_register.get(), 24);
    }

    #[test]
    fn test_addr_register_update() {
        let mut addr_register = AddrRegister::default();
        addr_register.set(12);
        addr_register.update(0b0001_1000); // Set Low Byte 24
        assert_eq!(addr_register.get(), 24);

        addr_register.update(24); // Set High Byte 0b0001_1000_0001_1000
        assert_eq!(addr_register.get(), 6168);
    }
}
