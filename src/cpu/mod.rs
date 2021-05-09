mod instructions;
mod processor_status;

use super::bus::MemoryBus;
use instructions::{get_instruction_from_opcode, Instruction, InstructionType};
use processor_status::ProcesssorStatus;

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
        Self {
            program_counter: 0,
            stack_pointer: 0,
            a: 0,
            x: 0,
            y: 0,
            processor_status: ProcesssorStatus::default(),
            bus: rom,
        }
    }

    pub fn read_next_byte(&mut self) -> u8 {
        let byte = self.bus.read_byte(self.program_counter);
        self.program_counter += 1;
        byte
    }

    pub fn start(&mut self) {
        loop {
            let instruction = get_instruction_from_opcode(self.read_next_byte());
            match instruction.instruction_type {
                InstructionType::LD => self.ld(instruction),
                InstructionType::BRK => return,
                InstructionType::TAX => self.tax(instruction),
                InstructionType::INC => self.inc(instruction),
                InstructionType::TODO => {
                    print!("Instruction needs implementing: {}", instruction)
                }
            }
        }
    }

    fn ld(&mut self, instruction: &Instruction) {
        if cfg!(debug_assertions) {
            println!("{}", instruction);
        }
        self.a = match instruction.op_code {
            0xa9 => self.read_next_byte(),
            _ => panic!("Unknown!"),
        };
        self.set_negative_and_zero_process_status(self.a);
    }

    fn tax(&mut self, instruction: &Instruction) {
        if cfg!(debug_assertions) {
            println!("{}", instruction);
        }
        self.x = self.a;
        self.set_negative_and_zero_process_status(self.x);
    }

    fn inc(&mut self, instruction: &Instruction) {
        if cfg!(debug_assertions) {
            println!("{}", instruction);
        }
        self.x = self.x.wrapping_add(1);
        self.set_negative_and_zero_process_status(self.x);
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
        return MemoryBus::new(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);
    }

    #[test]
    fn test_end_to_end() {
        let mut cpu = CPU::new(fake_rom());
        cpu.start();

        assert_eq!(cpu.x, 0xc1)
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new(MemoryBus::new(vec![0xe8, 0xe8, 0x00]));
        cpu.x = 0xff;
        cpu.start();

        assert_eq!(cpu.x, 1);
    }

    #[test]
    fn test_read_next_byte() {
        let mut cpu = CPU::new(fake_rom());
        let byte = cpu.read_next_byte();
        assert_eq!(byte, 0xa9);
        assert_eq!(cpu.program_counter, 0x01);

        let byte = cpu.read_next_byte();
        assert_eq!(byte, 0xc0);
        assert_eq!(cpu.program_counter, 0x02);
    }

    #[test]
    fn test_ld() {
        let mut cpu = CPU::new(MemoryBus::new(vec![0xa9, 0xc5, 0x00]));
        cpu.start();

        assert_eq!(cpu.a, 0xc5);
        assert_eq!(cpu.processor_status.get_zero(), false);
        assert_eq!(cpu.processor_status.get_negative(), true);
    }

    #[test]
    fn test_ld_zero() {
        let mut cpu = CPU::new(MemoryBus::new(vec![0xa9, 0x00, 0x00]));
        cpu.start();
        assert_eq!(cpu.processor_status.get_zero(), true)
    }

    #[test]
    fn test_tax_zero() {
        let mut cpu = CPU::new(MemoryBus::new(vec![0xaa, 0x00]));
        cpu.a = 0x00;
        cpu.start();
        assert_eq!(cpu.x, cpu.a);
        assert_eq!(cpu.processor_status.get_zero(), true)
    }

    #[test]
    fn test_tax() {
        let mut cpu = CPU::new(MemoryBus::new(vec![0xaa, 0x00]));
        cpu.a = 0x01;
        cpu.start();
        assert_eq!(cpu.x, cpu.a);
        assert_eq!(cpu.processor_status.get_zero(), false)
    }
}
