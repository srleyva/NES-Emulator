pub const RESET_VECTOR: u16 = 0xfffc;
pub const RESET_VECTOR_END: u16 = 0xfffd;

pub const NMI_VECTOR: u16 = 0xfffa;
pub const NMI_VECTOR_END: u16 = 0xfffb;

pub const IRQ_BRK_VECTOR: u16 = 0xfffe;
pub const IRQ_BRK_VECTOR_END: u16 = 0xffff;

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum InterruptType {
    NMI,
    IRQ,
    BRK,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Interrupt {
    pub itype: InterruptType,
    pub vector_addr: u16,
    pub break_flag: bool,
    pub cpu_cycles: u8,
}

pub const NMI: Interrupt = Interrupt {
    itype: InterruptType::NMI,
    vector_addr: NMI_VECTOR,
    break_flag: false,
    cpu_cycles: 2,
};

pub const IRQ: Interrupt = Interrupt {
    itype: InterruptType::IRQ,
    vector_addr: IRQ_BRK_VECTOR,
    break_flag: false,
    cpu_cycles: 2,
};

pub const BRK: Interrupt = Interrupt {
    itype: InterruptType::BRK,
    vector_addr: IRQ_BRK_VECTOR,
    break_flag: true,
    cpu_cycles: 1,
};

impl std::fmt::Display for Interrupt {
    #[cfg(debug_assertions)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Interrupt {:?} VectorAddr {:#04X?}",
            self.itype, self.vector_addr
        )
    }

    #[cfg(not(debug_assertions))]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Interrupt {:?}", self.itype)
    }
}
