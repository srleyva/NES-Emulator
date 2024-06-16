pub mod instructions;
pub mod interrupt;
pub mod processor_status;

use core::panic;
use std::{
    fmt::Debug,
    ops::{BitAnd, BitOr},
};

use self::interrupt::{Interrupt, InterruptType, BRK, NMI};

use super::bus::MemoryBus;
use instructions::{
    get_instruction_from_opcode, Instruction, InstructionType, MemoryAdressingMode,
};
use processor_status::ProcessorStatus;
use sdl2::libc::printf;

const STACK: u16 = 0x0100;
const OPCODE_EXIT: u8 = 0xf4;

#[derive(Clone)]
pub struct CPU {
    pub program_counter: u16,
    pub stack_pointer: u8,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub processor_status: ProcessorStatus,
    pub bus: MemoryBus,
}

impl PartialEq for CPU {
    fn eq(&self, other: &Self) -> bool {
        return self.x == other.x
            && self.y == self.y
            && self.a == self.a
            && self.processor_status == other.processor_status;
    }
}

impl Eq for CPU {}

impl std::fmt::Display for CPU {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Registers: a=[{:#04X?}] x=[{:#04X?}] y=[{:#04X?}] StackPointer=[{:#04X?}] ProgramCounter=[{:#04X?} ProcessorStatus=[{}]]", self.a, self.x, self.y, self.stack_pointer, self.program_counter, self.processor_status)
    }
}

impl Debug for CPU {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Registers: a=[{:#04X?}] x=[{:#04X?}] y=[{:#04X?}] StackPointer=[{:#04X?}] ProgramCounter=[{:#04X?} ProcessorStatus=[{}]]", self.a, self.x, self.y, self.stack_pointer, self.program_counter, self.processor_status)
    }
}

impl CPU {
    pub fn new(rom: MemoryBus) -> Self {
        let mut cpu = Self::new_with_state(rom, 0x8000, 0xFD, 0, 0, 0, ProcessorStatus::default());
        cpu.reset_cpu();
        cpu
    }

    pub fn new_with_state(
        rom: MemoryBus,
        program_counter: u16,
        stack_pointer: u8,
        a: u8,
        x: u8,
        y: u8,
        processor_status: ProcessorStatus,
    ) -> Self {
        let cpu = Self {
            program_counter,
            stack_pointer,
            a,
            x,
            y,
            processor_status,
            bus: rom,
        };
        cpu
    }

    pub fn start_with_callback<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut CPU, &Instruction),
    {
        loop {
            let program_counter_state = self.program_counter;
            let instruction = get_instruction_from_opcode(self.read_next_byte() as usize);

            if let Some(nmi) = self.bus.poll_nmi_status() {
                match nmi {
                    InterruptType::NMI => self.interrupt(&NMI),
                    _ => panic!("non-nmi interrupt sent: {:?}", nmi),
                }
            }
            if cfg!(debug_assertions) {
                println!("INSTRUCTION: {:?}", instruction);
            }
            let cycles: u8 = match instruction.instruction_type {
                InstructionType::AND => self.and(instruction),
                InstructionType::ADC => self.adc(instruction),
                InstructionType::ASL => self.asl(instruction),
                InstructionType::BCC => self.bcc(instruction),
                InstructionType::BCS => self.bcs(instruction),
                InstructionType::BEQ => self.beq(instruction),
                InstructionType::BIT => self.bit(instruction),
                InstructionType::BMI => self.bmi(instruction),
                InstructionType::BNE => self.bne(instruction),
                InstructionType::BPL => self.bpl(instruction),
                InstructionType::BRK => self.brk(instruction),
                InstructionType::BVC => self.bvc(instruction),
                InstructionType::BVS => self.bvs(instruction),
                InstructionType::CLC => self.clc(instruction),
                InstructionType::CLD => self.cld(instruction),
                InstructionType::CLI => self.cli(instruction),
                InstructionType::CLV => self.clv(instruction),
                InstructionType::CMP => self.cmp(instruction),
                InstructionType::CPX => self.cpx(instruction),
                InstructionType::CPY => self.cpy(instruction),
                InstructionType::DEC => self.dec(instruction),
                InstructionType::DEX => self.dex(instruction),
                InstructionType::DEY => self.dey(instruction),
                InstructionType::EOR => self.eor(instruction),
                InstructionType::ISB => self.isb(instruction),
                InstructionType::INC => self.inc(instruction),
                InstructionType::INX => self.inx(instruction),
                InstructionType::INY => self.iny(instruction),
                InstructionType::JMP => self.jmp(instruction),
                InstructionType::JSR => self.jsr(instruction),
                InstructionType::LDA => self.lda(instruction),
                InstructionType::LDX => self.ldx(instruction),
                InstructionType::LDY => self.ldy(instruction),
                InstructionType::LSR => self.lsr(instruction),
                InstructionType::NOP => self.nop(instruction),
                InstructionType::ORA => self.ora(instruction),
                InstructionType::PHA => self.pha(instruction),
                InstructionType::PHP => self.php(instruction),
                InstructionType::PLA => self.pla(instruction),
                InstructionType::PLP => self.plp(instruction),
                InstructionType::ROL => self.rol(instruction),
                InstructionType::ROR => self.ror(instruction),
                InstructionType::RTI => self.rti(instruction),
                InstructionType::RTS => self.rts(instruction),
                InstructionType::STA => self.sta(instruction),
                InstructionType::SBC => self.sbc(instruction),
                InstructionType::SEC => self.sec(instruction),
                InstructionType::SED => self.sed(instruction),
                InstructionType::SEI => self.sei(instruction),
                InstructionType::STX => self.stx(instruction),
                InstructionType::STY => self.sty(instruction),
                InstructionType::TAX => self.tax(instruction),
                InstructionType::TSX => self.tsx(instruction),
                InstructionType::TXS => self.txs(instruction),
                InstructionType::TXA => self.txa(instruction),
                InstructionType::TAY => self.tay(instruction),
                InstructionType::TYA => self.tya(instruction),
                InstructionType::NotImplemented => {
                    if instruction.op_code == OPCODE_EXIT {
                        return;
                    }
                    println!("Not implemented! {}", instruction);
                    self.nop(instruction)
                }
            };

            callback(self, instruction);
            if self.processor_status.contains(ProcessorStatus::BREAK) {
                self.interrupt(&BRK);
            }

            self.bus.tick(cycles);
            if cfg!(debug_assertions) {
                println!("CPU: {}", self);
            }
            // if program_counter_state == self.program_counter {
            //     self.program_counter += (cycles - 1) as u16;
            // }
        }
    }

    fn interrupt(&mut self, interrupt: &Interrupt) {
        if self
            .processor_status
            .contains(ProcessorStatus::INTERRUPT_DISABLE)
            && !matches!(interrupt.itype, InterruptType::IRQ)
        {
            return;
        }
        // if interrupt disable flag is set and interrupt type is not IRQ
        println!("Handling Interrupt: {}", interrupt);
        self.push_word(self.program_counter);
        let mut flag = self.processor_status.clone();
        flag.insert(ProcessorStatus::BREAK);
        flag.insert(ProcessorStatus::BREAK2);

        self.push(flag.bits());
        self.processor_status
            .insert(ProcessorStatus::INTERRUPT_DISABLE);

        self.bus.tick(interrupt.cpu_cycles);
        self.program_counter = self.bus.read_word(interrupt.vector_addr);
    }

    /*
    Instructions
    */

    fn adc(&mut self, instruction: &Instruction) -> u8 {
        let (data, page_cross) = self.read_byte(&instruction.memory_addressing_mode);
        self.a = self.add(self.a, data);
        self.set_negative_and_zero_process_status(self.a);
        if page_cross {
            instruction.cycle + 1
        } else {
            instruction.cycle
        }
    }

    fn and(&mut self, instruction: &Instruction) -> u8 {
        let (data, page_cross) = self.read_byte(&instruction.memory_addressing_mode);

        self.a &= data;
        self.set_negative_and_zero_process_status(self.a);

        if page_cross {
            instruction.cycle + 1
        } else {
            instruction.cycle
        }
    }

    fn asl(&mut self, instruction: &Instruction) -> u8 {
        match instruction.memory_addressing_mode {
            MemoryAdressingMode::Accumulator => {
                self.processor_status.set_carry(self.a >> 7 == 1);
                self.a <<= 1;
                self.set_negative_and_zero_process_status(self.a);
            }
            MemoryAdressingMode::Immediate => panic!("immediate addressing not supported for lsr"),
            _ => {
                let (address, _page_cross) = self.get_address(&instruction.memory_addressing_mode);
                let mut data = self.bus.read_byte(address);
                self.processor_status.set_carry(data >> 7 == 1);
                data <<= 1;
                self.bus.write_byte(address, data);
                self.set_negative_and_zero_process_status(data);
            }
        }
        instruction.cycle
    }

    fn bcc(&mut self, instruction: &Instruction) -> u8 {
        self.branch(!self.processor_status.contains(ProcessorStatus::CARRY));
        instruction.cycle
    }

    fn bcs(&mut self, instruction: &Instruction) -> u8 {
        self.branch(self.processor_status.contains(ProcessorStatus::CARRY));
        instruction.cycle
    }

    fn beq(&mut self, instruction: &Instruction) -> u8 {
        self.branch(self.processor_status.contains(ProcessorStatus::ZERO));
        instruction.cycle
    }

    fn bmi(&mut self, instruction: &Instruction) -> u8 {
        self.branch(self.processor_status.contains(ProcessorStatus::NEGATIVE));
        instruction.cycle
    }

    fn bne(&mut self, instruction: &Instruction) -> u8 {
        self.branch(!self.processor_status.contains(ProcessorStatus::ZERO));
        instruction.cycle
    }

    fn bpl(&mut self, instruction: &Instruction) -> u8 {
        self.branch(!self.processor_status.contains(ProcessorStatus::NEGATIVE));
        instruction.cycle
    }

    fn bvc(&mut self, instruction: &Instruction) -> u8 {
        self.branch(!self.processor_status.contains(ProcessorStatus::OVERFLOW));
        instruction.cycle
    }

    fn bvs(&mut self, instruction: &Instruction) -> u8 {
        self.branch(self.processor_status.contains(ProcessorStatus::OVERFLOW));
        instruction.cycle
    }

    fn bit(&mut self, instruction: &Instruction) -> u8 {
        let (data, page_cross) = self.read_byte(&instruction.memory_addressing_mode);
        self.processor_status.set_zero(data & self.a == 0);
        self.processor_status.set_negative(data & 0b10000000 > 0);
        self.processor_status.set_overflow(data & 0b01000000 > 0);
        instruction.cycle
    }

    fn brk(&mut self, instruction: &Instruction) -> u8 {
        self.interrupt(&BRK);
        instruction.cycle
    }

    fn clc(&mut self, instruction: &Instruction) -> u8 {
        self.processor_status.set_carry(false);
        instruction.cycle
    }

    fn cld(&mut self, instruction: &Instruction) -> u8 {
        self.processor_status.set_decimal(false);
        instruction.cycle
    }

    fn cli(&mut self, instruction: &Instruction) -> u8 {
        self.processor_status.set_interrupt_disable(false);
        instruction.cycle
    }

    fn clv(&mut self, instruction: &Instruction) -> u8 {
        self.processor_status.set_overflow(false);
        instruction.cycle
    }

    fn cmp(&mut self, instruction: &Instruction) -> u8 {
        let (data, page_cross) = self.read_byte(&instruction.memory_addressing_mode);
        self.compare(self.a, data);
        if page_cross {
            instruction.cycle + 1
        } else {
            instruction.cycle
        }
    }

    fn cpx(&mut self, instruction: &Instruction) -> u8 {
        let (data, page_cross) = self.read_byte(&instruction.memory_addressing_mode);
        self.compare(self.x, data);
        instruction.cycle
    }

    fn cpy(&mut self, instruction: &Instruction) -> u8 {
        let (data, page_cross) = self.read_byte(&instruction.memory_addressing_mode);
        self.compare(self.y, data);
        instruction.cycle
    }

    fn dec(&mut self, instruction: &Instruction) -> u8 {
        let (addr, page_cross) = self.get_address(&instruction.memory_addressing_mode);
        let value = self.bus.read_byte(addr);
        let new_value = value.wrapping_sub(1);
        self.bus.write_byte(addr, new_value);
        self.set_negative_and_zero_process_status(new_value);
        instruction.cycle
    }

    fn dex(&mut self, instruction: &Instruction) -> u8 {
        self.x = self.x.wrapping_sub(1);
        self.set_negative_and_zero_process_status(self.x);
        instruction.cycle
    }

    fn dey(&mut self, instruction: &Instruction) -> u8 {
        self.y = self.y.wrapping_sub(1);
        self.set_negative_and_zero_process_status(self.y);
        instruction.cycle
    }

    fn eor(&mut self, instruction: &Instruction) -> u8 {
        let (a, page_cross) = self.read_byte(&instruction.memory_addressing_mode);
        self.a ^= a;
        self.set_negative_and_zero_process_status(self.a);
        if page_cross {
            instruction.cycle + 1
        } else {
            instruction.cycle
        }
    }

    fn inc(&mut self, instruction: &Instruction) -> u8 {
        let (addr, page_cross) = self.get_address(&instruction.memory_addressing_mode);
        let value = self.bus.read_byte(addr);
        let new_value = value.wrapping_add(1);
        self.bus.write_byte(addr, new_value);
        self.set_negative_and_zero_process_status(new_value);
        instruction.cycle
    }

    fn inx(&mut self, instruction: &Instruction) -> u8 {
        self.x = self.x.wrapping_add(1);
        self.set_negative_and_zero_process_status(self.x);
        instruction.cycle
    }

    fn iny(&mut self, instruction: &Instruction) -> u8 {
        self.y = self.y.wrapping_add(1);
        self.set_negative_and_zero_process_status(self.y);
        instruction.cycle
    }

    fn isb(&mut self, instruction: &Instruction) -> u8 {
        todo!("{:?}", instruction)
    }

    fn jmp(&mut self, instruction: &Instruction) -> u8 {
        let addr = match instruction.memory_addressing_mode {
            MemoryAdressingMode::Indirect => {
                /*
                An original 6502 has does not correctly fetch the target address
                if the indirect vector falls on a page boundary (e.g. $xxFF where
                xx is any value from $00 to $FF). In this case fetches the LSB from
                $xxFF as expected but takes the MSB from $xx00.
                This is fixed in some later chips like the 65SC02 so for compatibility always
                ensure the indirect vector is not at the end of the page.
                */
                let addr = self.read_next_word();
                if addr & 0x00ff == 0x00ff {
                    let lo = self.bus.read_byte(addr);
                    let hi = self.bus.read_byte(addr & 0xFF00);
                    (hi as u16) << 8 | (lo as u16)
                } else {
                    self.bus.read_word(addr)
                }
            }
            MemoryAdressingMode::Absolute => self.read_next_word(),
            _ => panic!("Not support for jmp"),
        };
        self.program_counter = addr;
        instruction.cycle
    }

    fn jsr(&mut self, instruction: &Instruction) -> u8 {
        // so that the pc is incremented appropratiely
        let (addr, page_cross) = self.get_address(&instruction.memory_addressing_mode);

        let return_point = self.program_counter - 1;
        self.push_word(return_point);

        self.program_counter = addr;
        instruction.cycle
    }

    fn lda(&mut self, instruction: &Instruction) -> u8 {
        let (a, page_cross) = self.read_byte(&instruction.memory_addressing_mode);
        self.a = a;
        self.set_negative_and_zero_process_status(self.a);
        if page_cross {
            instruction.cycle + 1
        } else {
            instruction.cycle
        }
    }

    fn ldx(&mut self, instruction: &Instruction) -> u8 {
        let (x, page_cross) = self.read_byte(&instruction.memory_addressing_mode);
        self.x = x;
        self.set_negative_and_zero_process_status(self.x);
        instruction.cycle
    }

    fn ldy(&mut self, instruction: &Instruction) -> u8 {
        let (y, page_cross) = self.read_byte(&instruction.memory_addressing_mode);
        self.y = y;
        self.set_negative_and_zero_process_status(self.y);
        instruction.cycle
    }

    fn lsr(&mut self, instruction: &Instruction) -> u8 {
        match instruction.memory_addressing_mode {
            MemoryAdressingMode::Accumulator => {
                self.processor_status.set_carry(self.a & 0b0000_0001 == 1);
                self.a >>= 1;
                self.set_negative_and_zero_process_status(self.a);
            }
            MemoryAdressingMode::Immediate => panic!("immediate addressing not supported for lsr"),
            _ => {
                let (address, _page_cross) = self.get_address(&instruction.memory_addressing_mode);
                let mut data = self.bus.read_byte(address);
                self.processor_status.set_carry(data & 0b0000_0001 == 1);
                data >>= 1;
                self.bus.write_byte(address, data);
                self.set_negative_and_zero_process_status(data);
            }
        }

        instruction.cycle
    }

    fn nop(&mut self, instruction: &Instruction) -> u8 {
        // Account for illegal instructions
        match instruction.memory_addressing_mode {
            MemoryAdressingMode::Implied => (),
            _ => (_, _) = self.read_byte(&instruction.memory_addressing_mode),
        }
        instruction.cycle
    }

    fn ora(&mut self, instruction: &Instruction) -> u8 {
        let (data, page_cross) = self.read_byte(&instruction.memory_addressing_mode);
        self.a |= data;
        self.set_negative_and_zero_process_status(self.a);
        if page_cross {
            instruction.cycle + 1
        } else {
            instruction.cycle
        }
    }

    fn pha(&mut self, instruction: &Instruction) -> u8 {
        self.push(self.a);
        instruction.cycle
    }

    fn php(&mut self, instruction: &Instruction) -> u8 {
        self.push(self.processor_status.bitor(ProcessorStatus::BREAK).bits());
        instruction.cycle
    }

    fn pla(&mut self, instruction: &Instruction) -> u8 {
        let data = self.pop();
        self.a = data;
        self.set_negative_and_zero_process_status(self.a);
        instruction.cycle
    }

    fn plp(&mut self, instruction: &Instruction) -> u8 {
        self.processor_status = ProcessorStatus::from_bits_truncate(self.pop());
        self.processor_status.remove(ProcessorStatus::BREAK);
        self.processor_status.insert(ProcessorStatus::BREAK2);
        instruction.bytes
    }

    fn rol(&mut self, instruction: &Instruction) -> u8 {
        match instruction.memory_addressing_mode {
            MemoryAdressingMode::Accumulator => {
                let carry = self.processor_status.contains(ProcessorStatus::CARRY);
                self.processor_status
                    .set_carry(self.a & 0b0100_0000 != 0b0100_0000);
                self.a <<= 1;
                if carry {
                    self.a |= 0b0000_0001
                }
                self.set_negative_and_zero_process_status(self.a);
            }
            MemoryAdressingMode::Immediate => panic!("immediate addressing not supported for ror"),
            _ => {
                let (address, _page_cross) = self.get_address(&instruction.memory_addressing_mode);
                let mut data = self.bus.read_byte(address);
                let carry = self.processor_status.contains(ProcessorStatus::CARRY);
                self.processor_status
                    .set_carry(data & 0b0100_0000 != 0b0100_0000);
                data <<= 1;
                if carry {
                    data |= 0b0000_0001
                }
                self.bus.write_byte(address, data);
                self.set_negative_and_zero_process_status(data);
            }
        }

        instruction.cycle
    }

    fn ror(&mut self, instruction: &Instruction) -> u8 {
        match instruction.memory_addressing_mode {
            MemoryAdressingMode::Accumulator => {
                let carry = self.processor_status.contains(ProcessorStatus::CARRY);
                self.processor_status
                    .set_carry(self.a & 0b0000_0001 == 0b0000_0001);
                self.a >>= 1;
                if carry {
                    self.a |= 0b1000_0000
                }
                self.set_negative_and_zero_process_status(self.a);
            }
            MemoryAdressingMode::Immediate => panic!("immediate addressing not supported for ror"),
            _ => {
                let (address, _page_cross) = self.get_address(&instruction.memory_addressing_mode);
                let mut data = self.bus.read_byte(address);
                let carry = self.processor_status.contains(ProcessorStatus::CARRY);
                self.processor_status
                    .set_carry(data & 0b0000_0001 == 0b0000_0001);
                data >>= 1;
                if carry {
                    data |= 0b1000_0000
                }
                self.bus.write_byte(address, data);
                self.set_negative_and_zero_process_status(data);
            }
        }
        instruction.cycle
    }

    fn rts(&mut self, instruction: &Instruction) -> u8 {
        self.program_counter = self.pop_word() + 1;
        instruction.cycle
    }

    fn rti(&mut self, instruction: &Instruction) -> u8 {
        self.processor_status = ProcessorStatus::from_bits_truncate(self.pop());
        self.processor_status.remove(ProcessorStatus::BREAK);
        self.processor_status.insert(ProcessorStatus::BREAK2);
        self.program_counter = self.pop_word();
        instruction.cycle
    }

    fn sbc(&mut self, instruction: &Instruction) -> u8 {
        let (data, page_cross) = self.read_byte(&instruction.memory_addressing_mode);
        self.a = self.add(self.a, (data as i8).wrapping_neg().wrapping_sub(1) as u8);
        self.set_negative_and_zero_process_status(self.a);
        if page_cross {
            instruction.cycle + 1
        } else {
            instruction.cycle
        }
    }

    fn sec(&mut self, instruction: &Instruction) -> u8 {
        self.processor_status.set_carry(true);
        instruction.cycle
    }

    fn sed(&mut self, instruction: &Instruction) -> u8 {
        self.processor_status.set_decimal(true);
        instruction.cycle
    }

    fn sei(&mut self, instruction: &Instruction) -> u8 {
        self.processor_status.set_interrupt_disable(true);
        instruction.cycle
    }

    fn sta(&mut self, instruction: &Instruction) -> u8 {
        self.write_byte(&instruction.memory_addressing_mode, self.a);
        instruction.cycle
    }

    fn stx(&mut self, instruction: &Instruction) -> u8 {
        self.write_byte(&instruction.memory_addressing_mode, self.x);
        instruction.cycle
    }

    fn sty(&mut self, instruction: &Instruction) -> u8 {
        self.write_byte(&instruction.memory_addressing_mode, self.y);
        instruction.cycle
    }

    fn tax(&mut self, instruction: &Instruction) -> u8 {
        self.x = self.a;
        self.set_negative_and_zero_process_status(self.x);
        instruction.cycle
    }

    fn tay(&mut self, instruction: &Instruction) -> u8 {
        self.y = self.a;
        self.set_negative_and_zero_process_status(self.y);
        instruction.cycle
    }

    fn tya(&mut self, instruction: &Instruction) -> u8 {
        self.a = self.y;
        self.set_negative_and_zero_process_status(self.a);
        instruction.cycle
    }

    fn txs(&mut self, instruction: &Instruction) -> u8 {
        self.stack_pointer = self.x;
        instruction.cycle
    }

    fn txa(&mut self, instruction: &Instruction) -> u8 {
        self.a = self.x;
        self.set_negative_and_zero_process_status(self.a);
        instruction.cycle
    }

    fn tsx(&mut self, instruction: &Instruction) -> u8 {
        self.x = self.stack_pointer;
        self.set_negative_and_zero_process_status(self.x);
        instruction.cycle
    }

    /*
    Addressing
    */

    fn read_byte(&mut self, memory_addressing_mode: &MemoryAdressingMode) -> (u8, bool) {
        let (byte, page_cross) = match memory_addressing_mode {
            MemoryAdressingMode::Accumulator => (self.a, false),
            MemoryAdressingMode::Immediate => (self.read_next_byte(), false),
            _ => {
                let (addr, page_cross) = self.get_address(memory_addressing_mode);
                (self.bus.read_byte(addr), page_cross)
            }
        };

        (byte, page_cross)
    }

    fn write_byte(&mut self, memory_addressing_mode: &MemoryAdressingMode, byte: u8) -> bool {
        if cfg!(debug_assertions) {
            println!(" Write Data: {:#04X?}", byte);
        }
        match memory_addressing_mode {
            MemoryAdressingMode::Accumulator => {
                self.a = byte;
                self.set_negative_and_zero_process_status(self.a);
                false
            }
            _ => {
                let (addr, page_cross) = self.get_address(memory_addressing_mode);
                self.bus.write_byte(addr, byte);
                page_cross
            }
        }
    }

    fn get_address(&mut self, memory_addressing_mode: &MemoryAdressingMode) -> (u16, bool) {
        let (addr, boundary_cross) = match memory_addressing_mode {
            MemoryAdressingMode::Absolute => self.absolute_address(),
            MemoryAdressingMode::AbsoluteX => self.absolute_x_address(),
            MemoryAdressingMode::AbsoluteY => self.absolute_y_address(),
            MemoryAdressingMode::ZeroPage => self.zero_page_address(),
            MemoryAdressingMode::ZeroPageX => self.zero_page_x_address(),
            MemoryAdressingMode::ZeroPageY => self.zero_page_y_address(),
            MemoryAdressingMode::IndirectX => self.indirect_x_address(),
            MemoryAdressingMode::IndirectY => self.indirect_y_address(),
            MemoryAdressingMode::Relative => panic!("Look up not supported for relative"),
            _ => panic!("Not Supported: {:?}", memory_addressing_mode),
        };

        (addr, boundary_cross)
    }

    fn absolute_address(&mut self) -> (u16, bool) {
        (self.read_next_word(), false)
    }

    fn absolute_x_address(&mut self) -> (u16, bool) {
        let (base_addr, _) = self.absolute_address();
        let addr = base_addr.wrapping_add(self.x as u16);
        (addr as u16, addr & 0x00FF != base_addr & 0x00FF)
    }

    fn absolute_y_address(&mut self) -> (u16, bool) {
        let base_addr = self.read_next_word();
        let addr = base_addr.wrapping_add(self.y as u16);
        (addr as u16, addr & 0x00FF != base_addr & 0x00FF)
    }

    fn zero_page_address(&mut self) -> (u16, bool) {
        (self.read_next_byte() as u16, false)
    }

    fn zero_page_x_address(&mut self) -> (u16, bool) {
        let base_addr = self.read_next_byte();
        let addr = base_addr.wrapping_add(self.x);
        (addr as u16, addr & 0x00FF != base_addr & 0x00FF)
    }

    fn zero_page_y_address(&mut self) -> (u16, bool) {
        let base_addr = self.read_next_byte();
        let addr = base_addr.wrapping_add(self.y);
        (addr as u16, addr & 0x00FF != base_addr & 0x00FF)
    }

    fn indirect_x_address(&mut self) -> (u16, bool) {
        let base = self.read_next_byte();
        let ptr: u8 = (base as u8).wrapping_add(self.x);
        let lo = self.bus.read_byte(ptr as u16);
        let hi = self.bus.read_byte(ptr.wrapping_add(1) as u16);
        ((hi as u16) << 8 | (lo as u16), false)
    }

    fn indirect_y_address(&mut self) -> (u16, bool) {
        let base = self.read_next_byte();
        let lo = self.bus.read_byte(base as u16);
        let hi = self.bus.read_byte((base as u8).wrapping_add(1) as u16);
        let deref_base = (hi as u16) << 8 | (lo as u16);
        let addr = deref_base.wrapping_add(self.y as u16);
        let page_boundary_crossed = deref_base & 0xFF00 != (addr & 0xFF00);
        return (addr, page_boundary_crossed);
    }

    /*
    Helpers
    */

    fn compare(&mut self, register: u8, value: u8) {
        self.processor_status.set_carry(register >= value);
        self.set_negative_and_zero_process_status(register.wrapping_sub(value))
    }

    fn branch(&mut self, jump: bool) {
        let offset = self.read_next_byte() as i8;
        if jump {
            self.program_counter = self.program_counter.wrapping_add(offset as u16);
        }
    }

    fn add(&mut self, reg_value: u8, data: u8) -> u8 {
        let sum = reg_value as u16
            + data as u16
            + (if self.processor_status.contains(ProcessorStatus::CARRY) {
                1
            } else {
                0
            }) as u16;

        self.processor_status.set_carry(sum > 0xff);
        let sum = sum as u8;
        self.processor_status
            .set_overflow((data ^ sum) & (sum ^ self.a) & 0x80 != 0);
        sum
    }

    fn push(&mut self, byte: u8) {
        self.bus
            .write_byte((STACK as u16) + self.stack_pointer as u16, byte);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    }

    fn push_word(&mut self, word: u16) {
        let hi = (word >> 8) as u8;
        let lo = (word & 0xff) as u8;
        self.push(hi);
        self.push(lo);
    }

    fn pop(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);

        let byte = self
            .bus
            .read_byte((STACK as u16) + self.stack_pointer as u16);
        byte
    }

    fn pop_word(&mut self) -> u16 {
        self.pop() as u16 | (self.pop() as u16) << 8
    }

    pub(crate) fn reset_cpu(&mut self) {
        self.processor_status = ProcessorStatus::default();
        self.a = 0x0;
        self.x = 0x0;
        self.y = 0x0;
        self.stack_pointer = 0xfd;
        self.program_counter = self.bus.read_word(0xFFFC); // Part of the NES Spec
    }

    fn read_next_byte(&mut self) -> u8 {
        let byte = self.bus.read_byte(self.program_counter);
        self.program_counter += 1;
        byte
    }

    fn read_next_word(&mut self) -> u16 {
        let word = self.bus.read_word(self.program_counter);
        self.program_counter += 2;
        word
    }

    fn set_negative_and_zero_process_status(&mut self, int: u8) {
        self.processor_status.set_zero(int == 0);
        self.processor_status
            .set_negative(ProcessorStatus::is_negative(int));
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use super::super::rom::{Mirroring, Rom};
    use super::*;

    fn fake_rom(game_code: Vec<u8>) -> MemoryBus {
        let mut prg_rom = vec![0; 0x8000];
        let game_code_len = game_code.len();
        prg_rom[0x00..game_code_len].copy_from_slice(&game_code);
        let rom = Rom {
            prg_rom,
            chr_rom: vec![],
            mapper: 0,
            screen_mirroring: Mirroring::Horizontal,
        };
        let mut bus = MemoryBus::new(rom);

        let brk_handler = vec![OPCODE_EXIT; 4];
        bus.write_interrupt_handler(InterruptType::BRK, 0x8000 - 0x10, brk_handler);
        bus
    }

    pub fn start(cpu: &mut CPU) {
        cpu.start_with_callback(|_, _| {});
    }

    #[test]
    fn test_end_to_end() {
        let game_code = vec![
            0x20, 0x06, 0x06, 0x20, 0x38, 0x06, 0x20, 0x0d, 0x06, 0x20, 0x2a, 0x06, 0x60, 0xa9,
            0x02, 0x85, 0x02, 0xa9, 0x04, 0x85, 0x03, 0xa9, 0x11, 0x85, 0x10, 0xa9, 0x10, 0x85,
            0x12, 0xa9, 0x0f, 0x85, 0x14, 0xa9, 0x04, 0x85, 0x11, 0x85, 0x13, 0x85, 0x15, 0x60,
            0xa5, 0xfe, 0x85, 0x00, 0xa5, 0xfe, 0x29, 0x03, 0x18, 0x69, 0x02, 0x85, 0x01, 0x60,
            0x20, 0x4d, 0x06, 0x20, 0x8d, 0x06, 0x20, 0xc3, 0x06, 0x20, 0x19, 0x07, 0x20, 0x20,
            0x07, 0x20, 0x2d, 0x07, 0x4c, 0x38, 0x06, 0xa5, 0xff, 0xc9, 0x77, 0xf0, 0x0d, 0xc9,
            0x64, 0xf0, 0x14, 0xc9, 0x73, 0xf0, 0x1b, 0xc9, 0x61, 0xf0, 0x22, 0x60, 0xa9, 0x04,
            0x24, 0x02, 0xd0, 0x26, 0xa9, 0x01, 0x85, 0x02, 0x60, 0xa9, 0x08, 0x24, 0x02, 0xd0,
            0x1b, 0xa9, 0x02, 0x85, 0x02, 0x60, 0xa9, 0x01, 0x24, 0x02, 0xd0, 0x10, 0xa9, 0x04,
            0x85, 0x02, 0x60, 0xa9, 0x02, 0x24, 0x02, 0xd0, 0x05, 0xa9, 0x08, 0x85, 0x02, 0x60,
            0x60, 0x20, 0x94, 0x06, 0x20, 0xa8, 0x06, 0x60, 0xa5, 0x00, 0xc5, 0x10, 0xd0, 0x0d,
            0xa5, 0x01, 0xc5, 0x11, 0xd0, 0x07, 0xe6, 0x03, 0xe6, 0x03, 0x20, 0x2a, 0x06, 0x60,
            0xa2, 0x02, 0xb5, 0x10, 0xc5, 0x10, 0xd0, 0x06, 0xb5, 0x11, 0xc5, 0x11, 0xf0, 0x09,
            0xe8, 0xe8, 0xe4, 0x03, 0xf0, 0x06, 0x4c, 0xaa, 0x06, 0x4c, 0x35, 0x07, 0x60, 0xa6,
            0x03, 0xca, 0x8a, 0xb5, 0x10, 0x95, 0x12, 0xca, 0x10, 0xf9, 0xa5, 0x02, 0x4a, 0xb0,
            0x09, 0x4a, 0xb0, 0x19, 0x4a, 0xb0, 0x1f, 0x4a, 0xb0, 0x2f, 0xa5, 0x10, 0x38, 0xe9,
            0x20, 0x85, 0x10, 0x90, 0x01, 0x60, 0xc6, 0x11, 0xa9, 0x01, 0xc5, 0x11, 0xf0, 0x28,
            0x60, 0xe6, 0x10, 0xa9, 0x1f, 0x24, 0x10, 0xf0, 0x1f, 0x60, 0xa5, 0x10, 0x18, 0x69,
            0x20, 0x85, 0x10, 0xb0, 0x01, 0x60, 0xe6, 0x11, 0xa9, 0x06, 0xc5, 0x11, 0xf0, 0x0c,
            0x60, 0xc6, 0x10, 0xa5, 0x10, 0x29, 0x1f, 0xc9, 0x1f, 0xf0, 0x01, 0x60, 0x4c, 0x35,
            0x07, 0xa0, 0x00, 0xa5, 0xfe, 0x91, 0x00, 0x60, 0xa6, 0x03, 0xa9, 0x00, 0x81, 0x10,
            0xa2, 0x00, 0xa9, 0x01, 0x81, 0x10, 0x60, 0xa2, 0x00, 0xea, 0xea, 0xca, 0xd0, 0xfb,
            0x60,
        ];
        let mut cpu = CPU::new(fake_rom(game_code));
        start(&mut cpu);
    }

    #[test]
    fn test_adc() {
        let mut cpu = CPU::new(fake_rom(vec![0x69, 0x10, 0x00]));
        // Set the ROM start to default
        cpu.program_counter = 0x8000;
        cpu.a = 0x00;
        start(&mut cpu);

        assert_eq!(cpu.a, 0x10);
        assert!(!cpu.processor_status.contains(ProcessorStatus::CARRY));
        assert!(!cpu.processor_status.contains(ProcessorStatus::OVERFLOW));
    }

    #[test]
    fn test_adc_carry() {
        let mut cpu = CPU::new(fake_rom(vec![0x69, 0x10, 0x00]));
        // Set the ROM start to default
        cpu.program_counter = 0x8000;
        cpu.a = 0xff;
        start(&mut cpu);

        assert_eq!(cpu.a, 15);
        assert!(cpu.processor_status.contains(ProcessorStatus::CARRY));
        assert!(!cpu.processor_status.contains(ProcessorStatus::OVERFLOW));
    }

    #[test]
    fn test_asl() {
        let mut cpu = CPU::new(fake_rom(vec![0x0a, 0x00]));

        cpu.program_counter = 0x8000;
        cpu.a = 0b1111_1111;
        start(&mut cpu);

        assert_eq!(cpu.a, 0b1111_1110);
        assert!(cpu.processor_status.contains(ProcessorStatus::CARRY));
    }

    #[test]
    fn test_asl_no_carry() {
        let mut cpu = CPU::new(fake_rom(vec![0x0a, 0x00]));

        cpu.program_counter = 0x8000;
        cpu.a = 0b0111_1111;
        start(&mut cpu);

        assert_eq!(cpu.a, 0b1111_1110);
        assert!(!cpu.processor_status.contains(ProcessorStatus::CARRY));
    }

    #[test]
    fn test_bcc_carry() {
        let mut cpu = CPU::new(fake_rom(vec![0x90, 0x02, 0x69, 0x01, 0x69, 0x01, 0x00]));

        cpu.program_counter = 0x8000;
        cpu.processor_status.set_carry(true);
        start(&mut cpu);
        assert_eq!(cpu.a, 0x03) // Carry is set so...it adds with a carry
    }

    #[test]
    fn test_bcc_no_carry() {
        let mut cpu = CPU::new(fake_rom(vec![0x90, 0x02, 0x69, 0x01, 0x69, 0x01, 0x00]));

        cpu.program_counter = 0x8000;
        cpu.processor_status.set_carry(false);
        start(&mut cpu);
        assert_eq!(cpu.a, 0x01)
    }
    #[test]
    fn test_bcs_carry() {
        let mut cpu = CPU::new(fake_rom(vec![0xb0, 0x02, 0x69, 0x01, 0x69, 0x01, 0x00]));

        cpu.program_counter = 0x8000;
        cpu.processor_status.set_carry(true);
        start(&mut cpu);
        assert_eq!(cpu.a, 0x02) // Carry is set so...it adds with a carry
    }

    #[test]
    fn test_bcs_no_carry() {
        let mut cpu = CPU::new(fake_rom(vec![0xb0, 0x02, 0x69, 0x01, 0x69, 0x01, 0x00]));

        cpu.program_counter = 0x8000;
        cpu.processor_status.set_carry(false);
        start(&mut cpu);
        assert_eq!(cpu.a, 0x02)
    }

    #[test]
    fn test_bit_zero() {
        let mut cpu = CPU::new(fake_rom(vec![0x2c, 0xaa, 0x00]));
        cpu.program_counter = 0x8000;
        cpu.a = 0b0111_1111;
        cpu.bus.write_byte(0xaa, 0b0000_0000);
        start(&mut cpu);

        assert!(cpu.processor_status.contains(ProcessorStatus::ZERO));
        assert!(!cpu.processor_status.contains(ProcessorStatus::OVERFLOW));
        assert!(!cpu.processor_status.contains(ProcessorStatus::NEGATIVE));
    }

    #[test]
    fn test_bit_not_zero_overflow_carry() {
        let mut cpu = CPU::new(fake_rom(vec![0x2c, 0xaa, 0x00]));
        cpu.program_counter = 0x8000;
        cpu.a = 0b0111_1111;
        cpu.bus.write_byte(0xaa, 0b1100_0001);
        start(&mut cpu);

        assert!(!cpu.processor_status.contains(ProcessorStatus::ZERO));
        assert!(cpu.processor_status.contains(ProcessorStatus::OVERFLOW));
        assert!(cpu.processor_status.contains(ProcessorStatus::NEGATIVE));
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new(fake_rom(vec![0xe8, 0xe8, 0x00]));
        // Set the ROM start to default
        cpu.program_counter = 0x8000;
        cpu.x = 0xff;
        start(&mut cpu);

        assert_eq!(cpu.x, 1);
    }

    #[test]
    fn test_read_next_byte() {
        let mut cpu = CPU::new(fake_rom(vec![0x06, 0x12]));
        // Set the ROM start to default
        cpu.program_counter = 0x8000;

        let byte = cpu.read_next_byte();
        assert_eq!(byte, 0x06);
        assert_eq!(cpu.program_counter, 0x8001);

        let byte = cpu.read_next_byte();
        assert_eq!(byte, 0x12);
        assert_eq!(cpu.program_counter, 0x8002);
    }

    #[test]
    fn test_ld() {
        let mut cpu = CPU::new(fake_rom(vec![0xa9, 0xc5, 0x00]));
        // Set the ROM start to default
        cpu.program_counter = 0x8000;

        start(&mut cpu);

        assert_eq!(cpu.a, 0xc5);
        assert_eq!(cpu.processor_status.contains(ProcessorStatus::ZERO), false);
        assert_eq!(
            cpu.processor_status.contains(ProcessorStatus::NEGATIVE),
            true
        );
    }

    #[test]
    fn test_ld_from_memory() {
        let mut cpu = CPU::new(fake_rom(vec![0xa5, 0x10, 0x00]));
        cpu.bus.write_byte(0x10, 0x55);
        cpu.program_counter = 0x8000;

        start(&mut cpu);

        assert_eq!(cpu.a, 0x55);
    }

    #[test]
    fn test_ld_zero() {
        let mut cpu = CPU::new(fake_rom(vec![0xa9, 0x00, 0x00]));
        // Set the ROM start to default
        cpu.program_counter = 0x8000;

        start(&mut cpu);
        assert_eq!(cpu.processor_status.contains(ProcessorStatus::ZERO), true)
    }

    #[test]
    fn test_tax_zero() {
        let mut cpu = CPU::new(fake_rom(vec![0xaa, 0x00]));
        // Set the ROM start to default
        cpu.program_counter = 0x8000;

        cpu.a = 0x00;
        start(&mut cpu);
        assert_eq!(cpu.x, cpu.a);
        assert_eq!(cpu.processor_status.contains(ProcessorStatus::ZERO), true)
    }

    #[test]
    fn test_tax() {
        let mut cpu = CPU::new(fake_rom(vec![0xaa, 0x00]));
        // Set the ROM start to default
        cpu.program_counter = 0x8000;

        cpu.a = 0x01;
        start(&mut cpu);
        assert_eq!(cpu.x, cpu.a);
        assert_eq!(cpu.processor_status.contains(ProcessorStatus::ZERO), false)
    }

    #[test]
    fn test_sta() {
        let mut cpu = CPU::new(fake_rom(vec![0x85, 0x04, 0x00]));
        // Set the ROM start to default
        cpu.program_counter = 0x8000;

        cpu.a = 0x10;
        start(&mut cpu);

        assert_eq!(cpu.a, cpu.bus.read_byte(0x04));
    }

    #[test]

    fn test_stack() {
        let mut cpu = CPU::new(fake_rom(vec![0x85, 0x04, 0x00]));
        cpu.push(0x10);
        assert_eq!(cpu.pop(), 0x10);

        cpu.push_word(0xfff);
        assert_eq!(cpu.pop_word(), 0xfff);

        cpu.push(0x20);
        cpu.push(0x34);
        cpu.push(0x56);
        cpu.push(0x78);

        for i in vec![0x78, 0x56, 0x34, 0x20] {
            assert_eq!(cpu.pop(), i);
        }
    }
}
