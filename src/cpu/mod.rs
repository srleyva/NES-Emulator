pub mod instructions;
pub mod processor_status;

use super::bus::MemoryBus;
use instructions::{
    get_instruction_from_opcode, Instruction, InstructionType, MemoryAdressingMode,
};
use processor_status::ProcesssorStatus;

const STACK: u16 = 0x0100;

#[derive(Clone, Debug)]
pub struct CPU {
    program_counter: u16,
    stack_pointer: u8,
    a: u8,
    x: u8,
    y: u8,
    processor_status: ProcesssorStatus,
    pub(crate) bus: MemoryBus,
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

impl CPU {
    pub fn new(rom: MemoryBus) -> Self {
        let mut cpu = Self::new_with_state(rom, 0x8000, 0xFD, 0, 0, 0, ProcesssorStatus::default());
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
        processor_status: ProcesssorStatus,
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
            let instruction = get_instruction_from_opcode(self.read_next_byte());
            if cfg!(debug_assertions) {
                print!("{}", instruction);
            }
            match instruction.instruction_type {
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
                InstructionType::BRK => {
                    self.processor_status.set_break(true);
                }
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
                InstructionType::NotImplemented => panic!("Not implemented! {}", instruction),
            }
            if cfg!(debug_assertions) {
                println!();
                println!("CPU: {}", self);
            }
            callback(self, instruction);
            if self.processor_status.get_break() {
                return;
            }
        }
    }

    /*
    Instructions
    */

    fn adc(&mut self, instruction: &Instruction) {
        let data = self.read_byte(&instruction.memory_addressing_mode);
        self.a = self.add(self.a, data);
        self.set_negative_and_zero_process_status(self.a);
    }

    fn and(&mut self, instruction: &Instruction) {
        let data = self.read_byte(&instruction.memory_addressing_mode);

        self.a &= data;
        self.set_negative_and_zero_process_status(self.a)
    }

    fn asl(&mut self, instruction: &Instruction) {
        let mut data = self.read_byte(&instruction.memory_addressing_mode);
        self.processor_status.set_carry(data >> 7 == 1);
        data <<= 1;
        self.write_byte(&instruction.memory_addressing_mode, data);
        self.set_negative_and_zero_process_status(data);
    }

    fn bcc(&mut self, _instruction: &Instruction) {
        self.branch(!self.processor_status.get_carry());
    }

    fn bcs(&mut self, _instruction: &Instruction) {
        self.branch(self.processor_status.get_carry());
    }

    fn beq(&mut self, _instruction: &Instruction) {
        self.branch(self.processor_status.get_zero());
    }

    fn bmi(&mut self, _instruction: &Instruction) {
        self.branch(self.processor_status.get_negative());
    }

    fn bne(&mut self, _instruction: &Instruction) {
        self.branch(!self.processor_status.get_zero())
    }

    fn bpl(&mut self, _instruction: &Instruction) {
        self.branch(!self.processor_status.get_negative())
    }

    fn bvc(&mut self, _instruction: &Instruction) {
        self.branch(!self.processor_status.get_overflow())
    }

    fn bvs(&mut self, _instruction: &Instruction) {
        self.branch(self.processor_status.get_overflow())
    }

    fn bit(&mut self, instruction: &Instruction) {
        let data = self.read_byte(&instruction.memory_addressing_mode);
        self.processor_status.set_zero(data & self.a == 0);
        self.processor_status.set_negative(data & 0b10000000 > 0);
        self.processor_status.set_overflow(data & 0b01000000 > 0);
    }

    fn clc(&mut self, _instruction: &Instruction) {
        self.processor_status.set_carry(false);
    }

    fn cld(&mut self, _instruction: &Instruction) {
        self.processor_status.set_decimal(false);
    }

    fn cli(&mut self, _instruction: &Instruction) {
        self.processor_status.set_interrupt(false);
    }

    fn clv(&mut self, _instruction: &Instruction) {
        self.processor_status.set_overflow(false);
    }

    fn cmp(&mut self, instruction: &Instruction) {
        let data = self.read_byte(&instruction.memory_addressing_mode);
        self.compare(self.a, data);
    }

    fn cpx(&mut self, instruction: &Instruction) {
        let data = self.read_byte(&instruction.memory_addressing_mode);
        self.compare(self.x, data);
    }

    fn cpy(&mut self, instruction: &Instruction) {
        let data = self.read_byte(&instruction.memory_addressing_mode);
        self.compare(self.y, data);
    }

    fn dec(&mut self, instruction: &Instruction) {
        let addr = self.get_address(&instruction.memory_addressing_mode);
        let value = self.bus.read_byte(addr);
        let new_value = value.wrapping_sub(1);
        self.bus.write_byte(addr, new_value);
        self.set_negative_and_zero_process_status(new_value)
    }

    fn dex(&mut self, _instruction: &Instruction) {
        self.x = self.x.wrapping_sub(1);
        self.set_negative_and_zero_process_status(self.x);
    }

    fn dey(&mut self, _instruction: &Instruction) {
        self.y = self.y.wrapping_sub(1);
        self.set_negative_and_zero_process_status(self.y)
    }

    fn eor(&mut self, instruction: &Instruction) {
        self.a ^= self.read_byte(&instruction.memory_addressing_mode);
        self.set_negative_and_zero_process_status(self.a);
    }

    fn inc(&mut self, instruction: &Instruction) {
        let addr = self.get_address(&instruction.memory_addressing_mode);
        let value = self.bus.read_byte(addr);
        let new_value = value.wrapping_add(1);
        self.bus.write_byte(addr, new_value);
        self.set_negative_and_zero_process_status(new_value)
    }

    fn inx(&mut self, _instruction: &Instruction) {
        self.x = self.x.wrapping_add(1);
        self.set_negative_and_zero_process_status(self.x);
    }

    fn iny(&mut self, _instruction: &Instruction) {
        self.y = self.y.wrapping_add(1);
        self.set_negative_and_zero_process_status(self.y);
    }

    fn jmp(&mut self, instruction: &Instruction) {
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
    }

    fn jsr(&mut self, instruction: &Instruction) {
        // so that the pc is incremented appropratiely
        let addr = self.get_address(&instruction.memory_addressing_mode);

        let return_point = self.program_counter - 1;
        self.push_word(return_point);

        self.program_counter = addr;
    }

    fn lda(&mut self, instruction: &Instruction) {
        self.a = self.read_byte(&instruction.memory_addressing_mode);
        self.set_negative_and_zero_process_status(self.a);
    }

    fn ldx(&mut self, instruction: &Instruction) {
        self.x = self.read_byte(&instruction.memory_addressing_mode);
        self.set_negative_and_zero_process_status(self.x);
    }

    fn ldy(&mut self, instruction: &Instruction) {
        self.y = self.read_byte(&instruction.memory_addressing_mode);
        self.set_negative_and_zero_process_status(self.y);
    }

    fn lsr(&mut self, instruction: &Instruction) {
        let mut data = self.read_byte(&instruction.memory_addressing_mode);
        self.processor_status.set_carry(data & 0b0000_0001 == 1);
        data >>= 1;
        self.write_byte(&instruction.memory_addressing_mode, data);
        self.set_negative_and_zero_process_status(data);
    }

    fn nop(&mut self, _instruction: &Instruction) {
        self.program_counter = self.program_counter.wrapping_add(1);
    }

    fn ora(&mut self, instruction: &Instruction) {
        let data = self.read_byte(&instruction.memory_addressing_mode);
        self.a |= data;
        self.set_negative_and_zero_process_status(self.a);
    }

    fn pha(&mut self, _instruction: &Instruction) {
        self.push(self.a);
    }

    fn php(&mut self, _instruction: &Instruction) {
        self.push(self.processor_status.inner);
    }

    fn pla(&mut self, instruction: &Instruction) {
        let data = self.pop();
        self.write_byte(&instruction.memory_addressing_mode, data)
    }

    fn plp(&mut self, _instruction: &Instruction) {
        self.processor_status.inner = self.pop();
    }

    fn rol(&mut self, instruction: &Instruction) {
        let mut data = self.read_byte(&instruction.memory_addressing_mode);
        let carry = self.processor_status.get_carry();
        self.processor_status
            .set_carry(data & 0b0100_0000 == 0b0100_0000);
        data <<= 1;
        if carry {
            data |= 0b0000_0001
        }
        self.write_byte(&instruction.memory_addressing_mode, data);
    }

    fn ror(&mut self, instruction: &Instruction) {
        let mut data = self.read_byte(&instruction.memory_addressing_mode);
        let carry = self.processor_status.get_carry();
        self.processor_status
            .set_carry(data & 0b0000_0001 == 0b0000_0001);
        data >>= 1;
        if carry {
            data |= 0b1000_0000
        }
        self.write_byte(&instruction.memory_addressing_mode, data);
    }

    fn rts(&mut self, _instruction: &Instruction) {
        self.program_counter = self.pop_word() + 1;
    }

    fn rti(&mut self, _instruction: &Instruction) {
        self.processor_status.inner = self.pop();
        self.program_counter = self.pop_word();
    }

    fn sbc(&mut self, instruction: &Instruction) {
        let data = self.read_byte(&instruction.memory_addressing_mode);
        self.a = self.add(self.a, (data as i8).wrapping_neg().wrapping_sub(1) as u8)
    }

    fn sec(&mut self, _instruction: &Instruction) {
        self.processor_status.set_carry(true);
    }

    fn sed(&mut self, _instruction: &Instruction) {
        self.processor_status.set_decimal(true);
    }

    fn sei(&mut self, _instruction: &Instruction) {
        self.processor_status.set_interrupt(true);
    }

    fn sta(&mut self, instruction: &Instruction) {
        self.write_byte(&instruction.memory_addressing_mode, self.a);
    }

    fn stx(&mut self, instruction: &Instruction) {
        self.write_byte(&instruction.memory_addressing_mode, self.x);
    }

    fn sty(&mut self, instruction: &Instruction) {
        self.write_byte(&instruction.memory_addressing_mode, self.y);
    }

    fn tax(&mut self, _instruction: &Instruction) {
        self.x = self.a;
        self.set_negative_and_zero_process_status(self.x);
    }

    fn tay(&mut self, _instruction: &Instruction) {
        self.y = self.a;
        self.set_negative_and_zero_process_status(self.y);
    }

    fn tya(&mut self, _instruction: &Instruction) {
        self.a = self.y;
        self.set_negative_and_zero_process_status(self.a);
    }

    fn txs(&mut self, _instruction: &Instruction) {
        self.stack_pointer = self.x;
    }

    fn txa(&mut self, _instruction: &Instruction) {
        self.a = self.x;
        self.set_negative_and_zero_process_status(self.a);
    }

    fn tsx(&mut self, _instruction: &Instruction) {
        self.x = self.stack_pointer;
        self.set_negative_and_zero_process_status(self.x)
    }

    /*
    Addressing
    */

    fn read_byte(&mut self, memory_addressing_mode: &MemoryAdressingMode) -> u8 {
        let byte = match memory_addressing_mode {
            MemoryAdressingMode::Accumulator => self.a,
            MemoryAdressingMode::Immediate => self.read_next_byte(),
            _ => {
                let addr = self.get_address(memory_addressing_mode);
                self.bus.read_byte(addr)
            }
        };

        if cfg!(debug_assertions) {
            print!(" Read Data: {:#04X?}", byte);
        }

        byte
    }

    fn write_byte(&mut self, memory_addressing_mode: &MemoryAdressingMode, byte: u8) {
        if cfg!(debug_assertions) {
            print!(" Write Data: {:#04X?}", byte);
        }
        match memory_addressing_mode {
            MemoryAdressingMode::Accumulator => {
                self.a = byte;
                self.set_negative_and_zero_process_status(self.a)
            }
            _ => {
                let addr = self.get_address(memory_addressing_mode);
                self.bus.write_byte(addr, byte);
            }
        }
    }

    fn get_address(&mut self, memory_addressing_mode: &MemoryAdressingMode) -> u16 {
        let addr = match memory_addressing_mode {
            MemoryAdressingMode::Absolute => self.absolute_address(),
            MemoryAdressingMode::AbsoluteX => self.absolute_x_address(),
            MemoryAdressingMode::AbsoluteY => self.absolute_y_address(),
            MemoryAdressingMode::ZeroPage => self.zero_page_address(),
            MemoryAdressingMode::ZeroPageX => self.zero_page_x_address(),
            MemoryAdressingMode::ZeroPageY => self.zero_page_y_address(),
            MemoryAdressingMode::IndirectX => self.indirect_x_address(),
            MemoryAdressingMode::IndirectY => self.indirect_y_address(),
            MemoryAdressingMode::Relative => panic!("Look up not supported for relative"),
            _ => panic!("Not Supported"),
        };
        if cfg!(debug_assertions) {
            print!(" Addr: {:#04X?}", addr)
        }

        addr
    }

    fn absolute_address(&mut self) -> u16 {
        self.read_next_word()
    }

    fn absolute_x_address(&mut self) -> u16 {
        self.absolute_address().wrapping_add(self.x as u16)
    }

    fn absolute_y_address(&mut self) -> u16 {
        self.read_next_word().wrapping_add(self.y as u16)
    }

    fn zero_page_address(&mut self) -> u16 {
        self.read_next_byte() as u16
    }

    fn zero_page_x_address(&mut self) -> u16 {
        self.read_next_byte().wrapping_add(self.x) as u16
    }

    fn zero_page_y_address(&mut self) -> u16 {
        self.read_next_byte().wrapping_add(self.y) as u16
    }

    fn indirect_x_address(&mut self) -> u16 {
        let base = self.read_next_byte();
        let ptr: u8 = (base as u8).wrapping_add(self.x);
        let lo = self.bus.read_byte(ptr as u16);
        let hi = self.bus.read_byte(ptr.wrapping_add(1) as u16);
        (hi as u16) << 8 | (lo as u16)
    }

    fn indirect_y_address(&mut self) -> u16 {
        let base = self.read_next_byte();
        let lo = self.bus.read_byte(base as u16);
        let hi = self.bus.read_byte((base as u8).wrapping_add(1) as u16);
        let deref_base = (hi as u16) << 8 | (lo as u16);
        deref_base.wrapping_add(self.y as u16)
    }

    /*
    Helpers
    */

    fn compare(&mut self, register: u8, value: u8) {
        if register >= value {
            self.processor_status.set_carry(true);
        }
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
            + (if self.processor_status.get_carry() {
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
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
        self.bus
            .write_byte((STACK as u16) + self.stack_pointer as u16, byte);
    }

    fn push_word(&mut self, word: u16) {
        let hi = (word >> 8) as u8;
        let lo = (word & 0xff) as u8;
        self.push(hi);
        self.push(lo);
    }

    fn pop(&mut self) -> u8 {
        let byte = self
            .bus
            .read_byte((STACK as u16) + self.stack_pointer as u16);
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        byte
    }

    fn pop_word(&mut self) -> u16 {
        self.pop() as u16 | (self.pop() as u16) << 8
    }

    pub(crate) fn reset_cpu(&mut self) {
        self.processor_status = ProcesssorStatus::default();
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
            .set_negative(ProcesssorStatus::is_negative(int));
    }
}

#[cfg(test)]
mod test {
    use super::super::rom::{Mirroring, Rom};
    use super::*;

    fn fake_rom(game_code: Vec<u8>) -> MemoryBus {
        let mut code = [0 as u8; 0x7FFF].to_vec();
        code[0x00..game_code.len()].copy_from_slice(&game_code);
        let rom = Rom {
            prg_rom: code,
            chr_rom: vec![],
            mapper: 0,
            screen_mirroring: Mirroring::Horizontal,
        };
        return MemoryBus::new(rom);
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
        assert!(!cpu.processor_status.get_carry());
        assert!(!cpu.processor_status.get_overflow());
    }

    #[test]
    fn test_adc_carry() {
        let mut cpu = CPU::new(fake_rom(vec![0x69, 0x10, 0x00]));
        // Set the ROM start to default
        cpu.program_counter = 0x8000;
        cpu.a = 0xff;
        start(&mut cpu);

        assert_eq!(cpu.a, 15);
        assert!(cpu.processor_status.get_carry());
        assert!(!cpu.processor_status.get_overflow());
    }

    #[test]
    fn test_asl() {
        let mut cpu = CPU::new(fake_rom(vec![0x0a, 0x00]));

        cpu.program_counter = 0x8000;
        cpu.a = 0b1111_1111;
        start(&mut cpu);

        assert_eq!(cpu.a, 0b1111_1110);
        assert!(cpu.processor_status.get_carry());
    }

    #[test]
    fn test_asl_no_carry() {
        let mut cpu = CPU::new(fake_rom(vec![0x0a, 0x00]));

        cpu.program_counter = 0x8000;
        cpu.a = 0b0111_1111;
        start(&mut cpu);

        assert_eq!(cpu.a, 0b1111_1110);
        assert!(!cpu.processor_status.get_carry());
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

        assert!(cpu.processor_status.get_zero());
        assert!(!cpu.processor_status.get_overflow());
        assert!(!cpu.processor_status.get_negative());
    }

    #[test]
    fn test_bit_not_zero_overflow_carry() {
        let mut cpu = CPU::new(fake_rom(vec![0x2c, 0xaa, 0x00]));
        cpu.program_counter = 0x8000;
        cpu.a = 0b0111_1111;
        cpu.bus.write_byte(0xaa, 0b1100_0001);
        start(&mut cpu);

        assert!(!cpu.processor_status.get_zero());
        assert!(cpu.processor_status.get_overflow());
        assert!(cpu.processor_status.get_negative());
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
        assert_eq!(cpu.processor_status.get_zero(), false);
        assert_eq!(cpu.processor_status.get_negative(), true);
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
        assert_eq!(cpu.processor_status.get_zero(), true)
    }

    #[test]
    fn test_tax_zero() {
        let mut cpu = CPU::new(fake_rom(vec![0xaa, 0x00]));
        // Set the ROM start to default
        cpu.program_counter = 0x8000;

        cpu.a = 0x00;
        start(&mut cpu);
        assert_eq!(cpu.x, cpu.a);
        assert_eq!(cpu.processor_status.get_zero(), true)
    }

    #[test]
    fn test_tax() {
        let mut cpu = CPU::new(fake_rom(vec![0xaa, 0x00]));
        // Set the ROM start to default
        cpu.program_counter = 0x8000;

        cpu.a = 0x01;
        start(&mut cpu);
        assert_eq!(cpu.x, cpu.a);
        assert_eq!(cpu.processor_status.get_zero(), false)
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
