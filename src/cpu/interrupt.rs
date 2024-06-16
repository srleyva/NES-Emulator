#[derive(PartialEq, Eq, Clone, Debug)]
pub enum InterruptType {
    NMI,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Interrupt {
    pub itype: InterruptType,
    pub vector_addr: u16,
    pub flag_mask: u8,
    pub cpu_cycles: u8,
}

pub const NMI: Interrupt = Interrupt {
    itype: InterruptType::NMI,
    vector_addr: 0xfffA,
    flag_mask: 0b00100000,
    cpu_cycles: 2,
};
