#[macro_export]
macro_rules! instruction {
    ($mnemonic:expr,$op_code:expr,$bytes:expr,$cycle:expr,$instruction_type:expr,$memory_addressing:expr) => {
        Instruction {
            #[cfg(debug_assertions)]
            mnemonic: $mnemonic,

            op_code: $op_code,
            bytes: $bytes,
            cycle: $cycle,
            instruction_type: $instruction_type,
            memory_addressing_mode: $memory_addressing,
        }
    };
}

mod instruction_set;

use instruction_set::INSTRUCTION_SET;

pub enum InstructionType {
    NotImplemented,
    ADC,
    AND,
    ASL,
    BCC,
    BCS,
    BEQ,
    BIT,
    BMI,
    BNE,
    BPL,
    BRK,
    BVC,
    BVS,
    CLC,
    CLD,
    CLI,
    CLV,
    CMP,
    CPX,
    CPY,
    DEC,
    DEX,
    DEY,
    EOR,
    INC,
    INX,
    INY,
    JMP,
    JSR,
    LDA,
    LDX,
    LDY,
    LSR,
    NOP,
    ORA,
    PHA,
    PHP,
    PLA,
    PLP,
    ROL,
    ROR,
    RTI,
    RTS,
    SBC,
    SEC,
    SED,
    SEI,
    STA,
    STX,
    STY,
    TAX,
    TAY,
    TSX,
    TXA,
    TXS,
    TYA,
}

pub enum MemoryAdressingMode {
    Immediate,
    Implied,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Indirect,
    IndirectX,
    IndirectY,
    Relative,
    Accumulator,
}

pub struct Instruction {
    #[cfg(debug_assertions)]
    pub mnemonic: &'static str,

    pub op_code: u8,
    pub bytes: u8,
    pub cycle: u8,
    pub instruction_type: InstructionType,
    pub memory_addressing_mode: MemoryAdressingMode,
}

impl std::fmt::Display for Instruction {
    #[cfg(debug_assertions)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Instruction {:#04X?}: {}", self.op_code, self.mnemonic)
    }

    #[cfg(not(debug_assertions))]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Instruction {:#04X?}", self.op_code)
    }
}

pub fn get_instruction_from_opcode(op_code: u8) -> &'static Instruction {
    &INSTRUCTION_SET[op_code as usize]
}
