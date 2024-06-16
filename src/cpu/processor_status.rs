use std::fmt::Display;

bitflags! {
    #[derive(Default)]
    // 00100100
    pub struct ProcessorStatus: u8 {
        const CARRY             = 0b00000001;
        const ZERO              = 0b00000010;
        const INTERRUPT_DISABLE = 0b00000100;
        const DECIMAL           = 0b00001000;
        const BREAK             = 0b00010000;
        const BREAK2            = 0b00100000;
        const OVERFLOW          = 0b01000000;
        const NEGATIVE          = 0b10000000;
    }
}

impl ProcessorStatus {
    pub fn new(
        carry: bool,
        zero: bool,
        interrupt_disable: bool,
        decimal: bool,
        brk: bool,
        brk2: bool,
        overflow: bool,
        negative: bool,
    ) -> Self {
        let mut status = Self::default();
        status.set_carry(carry);
        status.set_zero(zero);
        status.set_interrupt_disable(interrupt_disable);
        status.set_decimal(decimal);
        status.set_break(brk);
        status.set_break2(brk2);
        status.set_overflow(overflow);
        status.set_negative(negative);
        status
    }

    pub fn set_break(&mut self, brk: bool) {
        if brk {
            self.insert(Self::BREAK)
        } else {
            self.remove(Self::BREAK)
        }
    }

    pub fn set_break2(&mut self, brk: bool) {
        if brk {
            self.insert(Self::BREAK2)
        } else {
            self.remove(Self::BREAK2)
        }
    }

    pub fn set_carry(&mut self, carry: bool) {
        if carry {
            self.insert(Self::CARRY)
        } else {
            self.remove(Self::CARRY)
        }
    }

    pub fn set_zero(&mut self, zero: bool) {
        if zero {
            self.insert(Self::ZERO)
        } else {
            self.remove(Self::ZERO)
        }
    }

    pub fn set_negative(&mut self, negative: bool) {
        if negative {
            self.insert(Self::NEGATIVE)
        } else {
            self.remove(Self::NEGATIVE)
        }
    }

    pub fn set_overflow(&mut self, overflow: bool) {
        if overflow {
            self.insert(Self::OVERFLOW)
        } else {
            self.remove(Self::OVERFLOW)
        }
    }

    pub fn set_decimal(&mut self, decimal: bool) {
        if decimal {
            self.insert(Self::DECIMAL)
        } else {
            self.remove(Self::DECIMAL)
        }
    }

    pub fn set_interrupt_disable(&mut self, interrupt_disable: bool) {
        if interrupt_disable {
            self.insert(Self::INTERRUPT_DISABLE)
        } else {
            self.remove(Self::INTERRUPT_DISABLE)
        }
    }

    pub fn is_negative(int: u8) -> bool {
        int & 0b1000_0000 != 0
    }
}

impl Display for ProcessorStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
                        f,
                        "Status [{:x}]: Carry=[{}] Zero=[{}] interrupt_disabled=[{}] dec=[{}] break=[{}] break2=[{}] overflow=[{}] negative=[{}]",
                        self.bits(),
                        self.contains(Self::CARRY),
                        self.contains(Self::ZERO),
                        self.contains(Self::INTERRUPT_DISABLE),
                        self.contains(Self::DECIMAL),
                        self.contains(Self::BREAK),
                        self.contains(Self::BREAK2),
                        self.contains(Self::OVERFLOW),
                        self.contains(Self::NEGATIVE)
        )
    }
}
