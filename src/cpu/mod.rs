mod instructions;
mod processor_status;

use super::bus::MemoryBus;
use instructions::{
    get_instruction_from_opcode, Instruction, InstructionType, MemoryAdressingMode,
};
use processor_status::ProcesssorStatus;

const STACK: u16 = 0x0100;

pub struct CPU {
    program_counter: u16,
    stack_pointer: u8,
    a: u8,
    x: u8,
    y: u8,
    processor_status: ProcesssorStatus,
    bus: MemoryBus,
}

impl CPU {
    pub fn new(rom: MemoryBus) -> Self {
        let mut cpu = Self {
            program_counter: 0x8000,
            stack_pointer: 0xfd,
            a: 0,
            x: 0,
            y: 0,
            processor_status: ProcesssorStatus::default(),
            bus: rom,
        };

        cpu.reset_cpu();
        cpu
    }

    pub fn start(&mut self) {
        loop {
            let instruction = get_instruction_from_opcode(self.read_next_byte());
            match instruction.instruction_type {
                InstructionType::ADC => self.adc(instruction),
                InstructionType::ASL => self.asl(instruction),
                InstructionType::BCC => self.bcc(instruction),
                InstructionType::BCS => self.bcs(instruction),
                InstructionType::BEQ => self.beq(instruction),
                InstructionType::BIT => self.bit(instruction),
                InstructionType::BMI => self.bmi(instruction),
                InstructionType::BNE => self.bne(instruction),
                InstructionType::BPL => self.bpl(instruction),
                InstructionType::BRK => return,
                InstructionType::BVC => self.bvc(instruction),
                InstructionType::BVS => self.bvs(instruction),
                InstructionType::CLC => self.clc(instruction),
                InstructionType::CLD => self.cld(instruction),
                InstructionType::CLI => self.cli(instruction),
                InstructionType::CLV => self.clv(instruction),
                InstructionType::CMP => self.cmp(instruction),
                InstructionType::LDA => self.lda(instruction),
                InstructionType::LDX => self.ldx(instruction),
                InstructionType::LDY => self.ldy(instruction),
                InstructionType::INX => self.inx(instruction),
                InstructionType::STA => self.sta(instruction),
                InstructionType::TAX => self.tax(instruction),
                InstructionType::TSX => self.tsx(instruction),
                InstructionType::TXS => self.txs(instruction),
                InstructionType::AND => self.and(instruction),
                InstructionType::NotImplemented => panic!("BAD Instruction"),
                _ => println!("Instruction: {} not implemented", instruction),
            }
        }
    }

    /*
    Instructions
    */

    fn adc(&mut self, instruction: &Instruction) {
        if cfg!(debug_assertions) {
            println!("{}", instruction);
        }
        let data = self.read_byte(&instruction.memory_addressing_mode);
        self.a = self.add(self.a, data);
        self.set_negative_and_zero_process_status(self.a);
    }

    fn and(&mut self, instruction: &Instruction) {
        if cfg!(debug_assertions) {
            println!("{}", instruction);
        }
        let data = self.read_byte(&instruction.memory_addressing_mode);

        self.a = data & self.a;
        self.set_negative_and_zero_process_status(self.a)
    }

    fn asl(&mut self, instruction: &Instruction) {
        if cfg!(debug_assertions) {
            println!("{}", instruction);
        }
        let mut data = self.read_byte(&instruction.memory_addressing_mode);
        self.processor_status.set_carry(data >> 7 == 1);
        data = data << 1;
        self.write_byte(&instruction.memory_addressing_mode, data);
    }

    fn bcc(&mut self, instruction: &Instruction) {
        if cfg!(debug_assertions) {
            println!("{}", instruction);
        }
        self.branch(!self.processor_status.get_carry());
    }

    fn bcs(&mut self, instruction: &Instruction) {
        if cfg!(debug_assertions) {
            println!("{}", instruction);
        }
        self.branch(self.processor_status.get_carry());
    }

    fn beq(&mut self, instruction: &Instruction) {
        if cfg!(debug_assertions) {
            println!("{}", instruction);
        }
        self.branch(self.processor_status.get_zero());
    }

    fn bmi(&mut self, instruction: &Instruction) {
        if cfg!(debug_assertions) {
            println!("{}", instruction);
        }
        self.branch(self.processor_status.get_negative());
    }

    fn bne(&mut self, instruction: &Instruction) {
        if cfg!(debug_assertions) {
            println!("{}", instruction);
        }
        self.branch(!self.processor_status.get_zero())
    }

    fn bpl(&mut self, instruction: &Instruction) {
        if cfg!(debug_assertions) {
            println!("{}", instruction);
        }
        self.branch(!self.processor_status.get_negative())
    }

    fn bvc(&mut self, instruction: &Instruction) {
        if cfg!(debug_assertions) {
            println!("{}", instruction);
        }
        self.branch(!self.processor_status.get_overflow())
    }

    fn bvs(&mut self, instruction: &Instruction) {
        if cfg!(debug_assertions) {
            println!("{}", instruction);
        }
        self.branch(self.processor_status.get_overflow())
    }

    fn bit(&mut self, instruction: &Instruction) {
        if cfg!(debug_assertions) {
            println!("{}", instruction);
        }
        let data = self.read_byte(&instruction.memory_addressing_mode);
        self.processor_status.set_zero(data & self.a == 0);
        self.processor_status.set_negative(data & 0b10000000 > 0);
        self.processor_status.set_overflow(data & 0b01000000 > 0);
    }

    fn clc(&mut self, instruction: &Instruction) {
        if cfg!(debug_assertions) {
            println!("{}", instruction);
        }
        self.processor_status.set_carry(false);
    }

    fn cld(&mut self, instruction: &Instruction) {
        if cfg!(debug_assertions) {
            println!("{}", instruction);
        }
        self.processor_status.set_decimal(false);
    }

    fn cli(&mut self, instruction: &Instruction) {
        if cfg!(debug_assertions) {
            println!("{}", instruction);
        }
        self.processor_status.set_interrupt(false);
    }

    fn clv(&mut self, instruction: &Instruction) {
        if cfg!(debug_assertions) {
            println!("{}", instruction);
        }
        self.processor_status.set_overflow(false);
    }

    fn cmp(&mut self, instruction: &Instruction) {
        if cfg!(debug_assertions) {
            println!("{}", instruction);
        }
        let data = self.read_byte(&instruction.memory_addressing_mode);
        self.processor_status.set_carry(self.a >= data);
        self.processor_status
            .set_zero(self.processor_status.get_zero());
    }

    fn lda(&mut self, instruction: &Instruction) {
        if cfg!(debug_assertions) {
            println!("{}", instruction);
        }
        self.a = self.read_byte(&instruction.memory_addressing_mode);
        self.set_negative_and_zero_process_status(self.a);
    }

    fn ldx(&mut self, instruction: &Instruction) {
        if cfg!(debug_assertions) {
            println!("{}", instruction);
        }
        self.x = self.read_byte(&instruction.memory_addressing_mode);
        self.set_negative_and_zero_process_status(self.x);
    }

    fn ldy(&mut self, instruction: &Instruction) {
        if cfg!(debug_assertions) {
            println!("{}", instruction);
        }
        self.y = self.read_byte(&instruction.memory_addressing_mode);
        self.set_negative_and_zero_process_status(self.y);
    }

    fn inx(&mut self, instruction: &Instruction) {
        if cfg!(debug_assertions) {
            println!("{}", instruction);
        }
        self.x = self.x.wrapping_add(1);
        self.set_negative_and_zero_process_status(self.x);
    }

    fn sta(&mut self, instruction: &Instruction) {
        if cfg!(debug_assertions) {
            println!("{}", instruction);
        }
        self.write_byte(&instruction.memory_addressing_mode, self.a);
    }

    fn tax(&mut self, instruction: &Instruction) {
        if cfg!(debug_assertions) {
            println!("{}", instruction);
        }
        self.x = self.a;
        self.set_negative_and_zero_process_status(self.x);
    }

    fn txs(&mut self, instruction: &Instruction) {
        if cfg!(debug_assertions) {
            println!("{}", instruction);
        }
        self.stack_pointer = self.x;
    }

    fn tsx(&mut self, instruction: &Instruction) {
        if cfg!(debug_assertions) {
            println!("{}", instruction);
        }
        self.x = self.stack_pointer;
    }

    /*
    Addressing
    */

    fn read_byte(&mut self, memory_addressing_mode: &MemoryAdressingMode) -> u8 {
        return match memory_addressing_mode {
            MemoryAdressingMode::Accumulator => self.a,
            _ => {
                let addr = self.get_address(memory_addressing_mode);
                return if addr == self.program_counter {
                    self.read_next_byte()
                } else {
                    self.bus.read_byte(addr)
                };
            }
        };
    }

    fn write_byte(&mut self, memory_addressing_mode: &MemoryAdressingMode, byte: u8) {
        match memory_addressing_mode {
            MemoryAdressingMode::Accumulator => {
                self.a = byte;
            }
            _ => {
                let addr = self.get_address(memory_addressing_mode);
                self.bus.write_byte(addr, byte);
            }
        }
    }

    fn get_address(&mut self, memory_addressing_mode: &MemoryAdressingMode) -> u16 {
        return match memory_addressing_mode {
            MemoryAdressingMode::Implied => self.program_counter,
            MemoryAdressingMode::Immediate => self.program_counter,
            MemoryAdressingMode::Absolute => self.absolute_address(),
            MemoryAdressingMode::AbsoluteX => self.absolute_x_address(),
            MemoryAdressingMode::AbsoluteY => self.absolute_y_address(),
            MemoryAdressingMode::ZeroPage => self.zero_page_address(),
            MemoryAdressingMode::ZeroPageX => self.zero_page_x_address(),
            MemoryAdressingMode::ZeroPageY => self.zero_page_y_address(),
            MemoryAdressingMode::Indirect => self.indirect_address(),
            MemoryAdressingMode::IndirectX => self.indirect_x_address(),
            MemoryAdressingMode::IndirectY => self.absolute_y_address(),
            MemoryAdressingMode::Relative => panic!("Look up not supported for relative"),
            MemoryAdressingMode::Accumulator => {
                panic!("This does not refer to memory but register a")
            }
        };
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
        self.zero_page_address().wrapping_add(self.x as u16) as u16
    }

    fn zero_page_y_address(&mut self) -> u16 {
        self.zero_page_address().wrapping_add(self.y as u16) as u16
    }

    fn indirect_address(&mut self) -> u16 {
        let ptr = self.absolute_address();
        self.bus.read_byte(ptr) as u16 | (self.bus.read_byte(ptr.wrapping_add(1)) as u16) << 8
    }

    fn indirect_x_address(&mut self) -> u16 {
        let ptr = self.zero_page_address().wrapping_add(self.x as u16);
        self.bus.read_byte(ptr) as u16 | (self.bus.read_byte(ptr.wrapping_add(1)) as u16) << 8
    }

    fn indirect_y_address(&mut self) -> u16 {
        let ptr = self.zero_page_address();
        let addr =
            self.bus.read_byte(ptr) as u16 | (self.bus.read_byte(ptr.wrapping_add(1)) as u16) << 8;
        addr.wrapping_add(self.y as u16)
    }

    /*
    Helpers
    */

    fn branch(&mut self, jump: bool) {
        let offset = self.read_next_byte() as u16;
        if jump {
            self.program_counter = self.program_counter.wrapping_add(offset);
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

    pub fn reset_cpu(&mut self) {
        self.processor_status = ProcesssorStatus::default();
        self.a = 0x0;
        self.x = 0x0;
        self.stack_pointer = 0xfd;
        self.program_counter = self.bus.read_word(0xFFFC); // Part of the NES Spec
    }

    pub fn read_next_byte(&mut self) -> u8 {
        let byte = self.bus.read_byte(self.program_counter);
        self.program_counter += 1;
        byte
    }

    pub fn read_next_word(&mut self) -> u16 {
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
    use super::*;

    fn fake_rom() -> MemoryBus {
        return MemoryBus::new(vec![
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
        ]);
    }

    #[test]
    #[ignore]
    fn test_end_to_end() {
        let mut cpu = CPU::new(fake_rom());
        cpu.program_counter = 0x8000;
        cpu.start();

        // assert_eq!(cpu.x, 0xc1)
    }

    #[test]
    fn test_adc() {
        let mut cpu = CPU::new(MemoryBus::new(vec![0x69, 0x10, 0x00]));
        // Set the ROM start to default
        cpu.program_counter = 0x8000;
        cpu.a = 0x00;
        cpu.start();

        assert_eq!(cpu.a, 0x10);
        assert!(!cpu.processor_status.get_carry());
        assert!(!cpu.processor_status.get_overflow());
    }

    #[test]
    fn test_adc_carry() {
        let mut cpu = CPU::new(MemoryBus::new(vec![0x69, 0x10, 0x00]));
        // Set the ROM start to default
        cpu.program_counter = 0x8000;
        cpu.a = 0xff;
        cpu.start();

        assert_eq!(cpu.a, 15);
        assert!(cpu.processor_status.get_carry());
        assert!(!cpu.processor_status.get_overflow());
    }

    #[test]
    fn test_asl() {
        let mut cpu = CPU::new(MemoryBus::new(vec![0x0a, 0x00]));

        cpu.program_counter = 0x8000;
        cpu.a = 0b1111_1111;
        cpu.start();

        assert_eq!(cpu.a, 0b1111_1110);
        assert!(cpu.processor_status.get_carry());
    }

    #[test]
    fn test_asl_no_carry() {
        let mut cpu = CPU::new(MemoryBus::new(vec![0x0a, 0x00]));

        cpu.program_counter = 0x8000;
        cpu.a = 0b0111_1111;
        cpu.start();

        assert_eq!(cpu.a, 0b1111_1110);
        assert!(!cpu.processor_status.get_carry());
    }

    #[test]
    fn test_bcc_carry() {
        let mut cpu = CPU::new(MemoryBus::new(vec![
            0x90, 0x02, 0x69, 0x01, 0x69, 0x01, 0x00,
        ]));

        cpu.program_counter = 0x8000;
        cpu.processor_status.set_carry(true);
        cpu.start();
        assert_eq!(cpu.a, 0x03) // Carry is set so...it adds with a carry
    }

    #[test]
    fn test_bcc_no_carry() {
        let mut cpu = CPU::new(MemoryBus::new(vec![
            0x90, 0x02, 0x69, 0x01, 0x69, 0x01, 0x00,
        ]));

        cpu.program_counter = 0x8000;
        cpu.processor_status.set_carry(false);
        cpu.start();
        assert_eq!(cpu.a, 0x01)
    }
    #[test]
    fn test_bcs_carry() {
        let mut cpu = CPU::new(MemoryBus::new(vec![
            0xb0, 0x02, 0x69, 0x01, 0x69, 0x01, 0x00,
        ]));

        cpu.program_counter = 0x8000;
        cpu.processor_status.set_carry(true);
        cpu.start();
        assert_eq!(cpu.a, 0x02) // Carry is set so...it adds with a carry
    }

    #[test]
    fn test_bcs_no_carry() {
        let mut cpu = CPU::new(MemoryBus::new(vec![
            0xb0, 0x02, 0x69, 0x01, 0x69, 0x01, 0x00,
        ]));

        cpu.program_counter = 0x8000;
        cpu.processor_status.set_carry(false);
        cpu.start();
        assert_eq!(cpu.a, 0x02)
    }

    #[test]
    fn test_bit_zero() {
        let mut cpu = CPU::new(MemoryBus::new(vec![0x2c, 0xaa, 0x00]));
        cpu.program_counter = 0x8000;
        cpu.a = 0b0111_1111;
        cpu.bus.write_byte(0xaa, 0b0000_0000);
        cpu.start();

        assert!(cpu.processor_status.get_zero());
        assert!(!cpu.processor_status.get_overflow());
        assert!(!cpu.processor_status.get_negative());
    }

    #[test]
    fn test_bit_not_zero_overflow_carry() {
        let mut cpu = CPU::new(MemoryBus::new(vec![0x2c, 0xaa, 0x00]));
        cpu.program_counter = 0x8000;
        cpu.a = 0b0111_1111;
        cpu.bus.write_byte(0xaa, 0b1100_0001);
        cpu.start();

        assert!(!cpu.processor_status.get_zero());
        assert!(cpu.processor_status.get_overflow());
        assert!(cpu.processor_status.get_negative());
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new(MemoryBus::new(vec![0xe8, 0xe8, 0x00]));
        // Set the ROM start to default
        cpu.program_counter = 0x8000;
        cpu.x = 0xff;
        cpu.start();

        assert_eq!(cpu.x, 1);
    }

    #[test]
    fn test_read_next_byte() {
        let mut cpu = CPU::new(MemoryBus::new(vec![0x06, 0x12]));
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
        let mut cpu = CPU::new(MemoryBus::new(vec![0xa9, 0xc5, 0x00]));
        // Set the ROM start to default
        cpu.program_counter = 0x8000;

        cpu.start();

        assert_eq!(cpu.a, 0xc5);
        assert_eq!(cpu.processor_status.get_zero(), false);
        assert_eq!(cpu.processor_status.get_negative(), true);
    }

    #[test]
    fn test_ld_from_memory() {
        let mut cpu = CPU::new(MemoryBus::new(vec![0xa5, 0x10, 0x00]));
        cpu.bus.write_byte(0x10, 0x55);
        cpu.program_counter = 0x8000;

        cpu.start();

        assert_eq!(cpu.a, 0x55);
    }

    #[test]
    fn test_ld_zero() {
        let mut cpu = CPU::new(MemoryBus::new(vec![0xa9, 0x00, 0x00]));
        // Set the ROM start to default
        cpu.program_counter = 0x8000;

        cpu.start();
        assert_eq!(cpu.processor_status.get_zero(), true)
    }

    #[test]
    fn test_tax_zero() {
        let mut cpu = CPU::new(MemoryBus::new(vec![0xaa, 0x00]));
        // Set the ROM start to default
        cpu.program_counter = 0x8000;

        cpu.a = 0x00;
        cpu.start();
        assert_eq!(cpu.x, cpu.a);
        assert_eq!(cpu.processor_status.get_zero(), true)
    }

    #[test]
    fn test_tax() {
        let mut cpu = CPU::new(MemoryBus::new(vec![0xaa, 0x00]));
        // Set the ROM start to default
        cpu.program_counter = 0x8000;

        cpu.a = 0x01;
        cpu.start();
        assert_eq!(cpu.x, cpu.a);
        assert_eq!(cpu.processor_status.get_zero(), false)
    }

    #[test]
    fn test_sta() {
        let mut cpu = CPU::new(MemoryBus::new(vec![0x85, 0x04, 0x00]));
        // Set the ROM start to default
        cpu.program_counter = 0x8000;

        cpu.a = 0x10;
        cpu.start();

        assert_eq!(cpu.a, cpu.bus.read_byte(0x04));
    }

    #[test]

    fn test_stack() {
        let mut cpu = CPU::new(MemoryBus::new(vec![0x85, 0x04, 0x00]));
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
