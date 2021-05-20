#[derive(Default)]
pub struct ProcesssorStatus {
    inner: u8,
}

impl std::fmt::Display for ProcesssorStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Carry=[{}] Zero=[{}] interrupt=[{}] dec=[{}] break=[{}] overflow=[{}] negative=[{}]",
            self.get_carry(),
            self.get_zero(),
            self.get_interrupt(),
            self.get_decimal(),
            self.get_break(),
            self.get_overflow(),
            self.get_negative()
        )
    }
}

impl ProcesssorStatus {
    /*
    0 - Carry
    1 - Zero
    2 - Interrupt
    3 - Decimal
    4 - Break
    5 - -
    6 - Overflow
    7 - Negative
    */
    pub fn get_carry(&self) -> bool {
        return self.inner & 0b0000_0001 == 0b0000_0001;
    }

    pub fn set_carry(&mut self, carry: bool) {
        if carry {
            self.inner = self.inner | 0b0000_0001
        } else {
            self.inner = self.inner & 0b1111_1110
        }
    }

    pub fn get_zero(&self) -> bool {
        return self.inner & 0b0000_0010 == 0b0000_0010;
    }

    pub fn set_zero(&mut self, zero: bool) {
        if zero {
            self.inner = self.inner | 0b0000_0010
        } else {
            self.inner = self.inner & 0b1111_1101
        }
    }

    pub fn get_interrupt(&self) -> bool {
        return self.inner & 0b0000_0100 == 0b0000_0100;
    }

    pub fn set_interrupt(&mut self, interrupt: bool) {
        if interrupt {
            self.inner = self.inner | 0b0000_0100
        } else {
            self.inner = self.inner & 0b1111_1011
        }
    }

    pub fn get_decimal(&self) -> bool {
        return self.inner & 0b0000_1000 == 0b0000_1000;
    }

    pub fn set_decimal(&mut self, decimal: bool) {
        if decimal {
            self.inner = self.inner | 0b0000_1000
        } else {
            self.inner = self.inner & 0b1111_0111
        }
    }

    pub fn get_break(&self) -> bool {
        return self.inner & 0b0001_0000 == 0b0001_0000;
    }

    pub fn set_break(&mut self, brk: bool) {
        if brk {
            self.inner = self.inner | 0b0001_0000
        } else {
            self.inner = self.inner & 0b1110_1111
        }
    }

    pub fn get_overflow(&self) -> bool {
        return self.inner & 0b0100_0000 == 0b0100_0000;
    }

    pub fn set_overflow(&mut self, overflow: bool) {
        if overflow {
            self.inner = self.inner | 0b0100_0000
        } else {
            self.inner = self.inner & 0b1011_1111
        }
    }

    pub fn get_negative(&self) -> bool {
        return self.inner & 0b1000_0000 == 0b1000_0000;
    }

    pub fn set_negative(&mut self, brk: bool) {
        if brk {
            self.inner = self.inner | 0b1000_0000
        } else {
            self.inner = self.inner & 0b0111_1111
        }
    }

    pub fn is_negative(int: u8) -> bool {
        int & 0b1000_0000 != 0
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_processer_status_carry() {
        let mut processor_status = ProcesssorStatus::default();
        assert_eq!(processor_status.get_carry(), false);

        processor_status.set_carry(true);
        assert_eq!(processor_status.get_carry(), true);

        processor_status.set_carry(false);
        assert_eq!(processor_status.get_carry(), false);
    }

    #[test]
    fn test_processer_status_zero() {
        let mut processor_status = ProcesssorStatus::default();
        assert_eq!(processor_status.get_zero(), false);

        processor_status.set_zero(true);
        assert_eq!(processor_status.get_zero(), true);

        processor_status.set_zero(false);
        assert_eq!(processor_status.get_zero(), false);
    }

    #[test]
    fn test_processer_status_interrupt() {
        let mut processor_status = ProcesssorStatus::default();
        assert_eq!(processor_status.get_interrupt(), false);

        processor_status.set_interrupt(true);
        assert_eq!(processor_status.get_interrupt(), true);

        processor_status.set_interrupt(false);
        assert_eq!(processor_status.get_interrupt(), false);
    }

    #[test]
    fn test_processer_status_decimal() {
        let mut processor_status = ProcesssorStatus::default();
        assert_eq!(processor_status.get_decimal(), false);

        processor_status.set_decimal(true);
        assert_eq!(processor_status.get_decimal(), true);

        processor_status.set_decimal(false);
        assert_eq!(processor_status.get_decimal(), false);
    }

    #[test]
    fn test_processer_status_break() {
        let mut processor_status = ProcesssorStatus::default();
        assert_eq!(processor_status.get_break(), false);

        processor_status.set_break(true);
        assert_eq!(processor_status.get_break(), true);

        processor_status.set_break(false);
        assert_eq!(processor_status.get_break(), false);
    }

    #[test]
    fn test_processer_status_overflow() {
        let mut processor_status = ProcesssorStatus::default();
        assert_eq!(processor_status.get_overflow(), false);

        processor_status.set_overflow(true);
        assert_eq!(processor_status.get_overflow(), true);

        processor_status.set_overflow(false);
        assert_eq!(processor_status.get_overflow(), false);
    }

    #[test]
    fn test_processer_status_negative() {
        let mut processor_status = ProcesssorStatus::default();
        assert_eq!(processor_status.get_negative(), false);

        processor_status.set_negative(true);
        assert_eq!(processor_status.get_negative(), true);

        processor_status.set_negative(false);
        assert_eq!(processor_status.get_negative(), false);
    }

    #[test]
    fn test_processer_status_is_negative() {
        let pos_int: u8 = 0b0000_0001;
        assert_eq!(ProcesssorStatus::is_negative(pos_int), false);

        let neg_int: u8 = 0b1000_0001;
        assert_eq!(ProcesssorStatus::is_negative(neg_int), true);
    }
}
