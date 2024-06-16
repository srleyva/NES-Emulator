#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct Address {
    value: (u8, u8),
    pointer: bool, // True is low pointer, False is high pointer
}

impl From<u16> for Address {
    fn from(address: u16) -> Self {
        let mut addr = Address::default();
        addr.set(address);
        addr
    }
}

impl Address {
    pub fn set(&mut self, address: u16) {
        self.value.0 = (address >> 8) as u8;
        self.value.1 = address as u8;
    }

    pub fn get(&self) -> u16 {
        (self.value.0 as u16) << 8 | (self.value.1 as u16)
    }

    pub fn increment(&mut self, value: u8) {
        let lo = self.value.1;
        self.value.1 = self.value.1.wrapping_add(value);
        if lo > self.value.1 {
            self.value.0 = self.value.0.wrapping_add(1);
        }
        self.set_mirror_down()
    }

    pub fn update(&mut self, data: u8) {
        if !self.pointer {
            self.value.0 = data;
        } else {
            self.value.1 = data;
        }
        self.set_mirror_down();
        self.pointer = !self.pointer;
    }

    fn set_mirror_down(&mut self) {
        if self.get() > 0x3fff {
            self.set(self.get() & 0b11111111111111);
        }
    }

    pub fn reset_latch(&mut self) {
        self.pointer = false
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_set() {
        let address = Address::from(0b0011_0000_0001_0110);
        assert_eq!(address.value.0, 0b0011_0000);
        assert_eq!(address.value.1, 0b0001_0110);
    }

    #[test]
    fn test_get() {
        let address = Address::from(0b0011_0000_0001_0110);
        assert_eq!(address.get(), 0b0011_0000_0001_0110);
    }

    #[test]
    fn test_increment() {
        let mut address = Address::from(0x3016);
        address.increment(1);

        assert_eq!(address.get(), 0x3017);

        address.increment(3);
        assert_eq!(address.get(), 0x301A);
    }

    #[test]
    fn test_update() {
        let mut address = Address::from(0x3016);
        address.update(0x20);
        address.update(0x06);

        assert_eq!(address.get(), 0x2006);
    }
}
