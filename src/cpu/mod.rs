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
        let mut cpu = Self {
            program_counter: 0x8000,
            stack_pointer: 0,
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
                InstructionType::LDA => self.lda(instruction),
                InstructionType::LDX => self.ldx(instruction),
                InstructionType::LDY => self.ldy(instruction),
                InstructionType::BRK => return,
                InstructionType::TAX => self.tax(instruction),
                InstructionType::INC => self.inc(instruction),
                InstructionType::STA => self.sta(instruction),
                InstructionType::TODO => println!("Instruction needs implementing: {}", instruction),
            }
        }
    }

    /*
    Instructions
    */

    fn lda(&mut self, instruction: &Instruction) {
        if cfg!(debug_assertions) {
            println!("{}", instruction);
        }
        let addr = match instruction.op_code {
            0xa9 => self.program_counter,
            0xa5 => self.zero_page_address(),
            0xb5 => self.zero_page_x_address(),
            0xad => self.absolute_address(),
            0xbd => self.absolute_x_address(),
            0xb9 => self.absolute_y_address(),
            0xa1 => self.indirect_x_address(),
            0xb1 => self.indirect_y_address(),
            _ => panic!("Unknown OpCode: {}", instruction.op_code),
        };

        if addr == self.program_counter {
           self.a = self.read_next_byte(); 
        } else {
            self.a = self.bus.read_byte(addr);
        }

        self.set_negative_and_zero_process_status(self.a);
    }

    fn ldx(&mut self, instruction: &Instruction) {
        if cfg!(debug_assertions) {
            println!("{}", instruction);
        }
        let addr = match instruction.op_code {
            0xa2 => self.program_counter,
            0xa6 => self.zero_page_address(),
            0xb6 => self.zero_page_y_address(),
            0xae => self.absolute_address(),
            0xbe => self.absolute_y_address(),
            _ => panic!("Unknown OpCode: {}", instruction.op_code),
        };

        if addr == self.program_counter {
           self.x = self.read_next_byte(); 
        } else {
            self.x = self.bus.read_byte(addr);
        }

        self.set_negative_and_zero_process_status(self.a);
    }

    fn ldy(&mut self, instruction: &Instruction) {
        if cfg!(debug_assertions) {
            println!("{}", instruction);
        }
        let addr = match instruction.op_code {
            0xa0 => self.program_counter,
            0xa4 => self.zero_page_address(),
            0xb4 => self.zero_page_x_address(),
            0xac => self.absolute_address(),
            0xbc => self.absolute_x_address(),
            _ => panic!("Unknown OpCode: {}", instruction.op_code),
        };

        if addr == self.program_counter {
           self.y = self.read_next_byte(); 
        } else {
            self.y = self.bus.read_byte(addr);
        }

        self.set_negative_and_zero_process_status(self.y);
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

    fn sta(&mut self, instruction: &Instruction) {
        if cfg!(debug_assertions) {
            println!("{}", instruction);
        }
        let addr = match instruction.op_code {
            0x85 => self.zero_page_address(),
            0x95 => self.zero_page_x_address(),
            0x8d => self.absolute_address(),
            0x9d => self.absolute_x_address(),
            0x99 => self.absolute_y_address(),
            0x81 => self.indirect_x_address(),
            0x91 => self.indirect_y_address(),
            _ => panic!("Unknown opcode for {}", instruction.op_code)
        };

        self.bus.write_byte(addr, self.a);
    }

    /*
    Addressing
    Lots of wet code, could be better dried by absracting the addressing and value loading seperately
    */

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
        let addr = self.bus.read_byte(ptr) as u16 | (self.bus.read_byte(ptr.wrapping_add(1)) as u16) << 8;
        addr.wrapping_add(self.y as u16)
    }

    /*
    Helpers
    */

    pub fn reset_cpu(&mut self) {
        self.processor_status = ProcesssorStatus::default();
        self.a = 0x0;
        self.x = 0x0;
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
            0x20, 0x06, 0x06, 0x20, 0x38, 0x06, 0x20, 0x0d, 0x06, 0x20, 0x2a, 0x06, 0x60, 0xa9, 0x02, 0x85,
            0x02, 0xa9, 0x04, 0x85, 0x03, 0xa9, 0x11, 0x85, 0x10, 0xa9, 0x10, 0x85, 0x12, 0xa9, 0x0f, 0x85,
            0x14, 0xa9, 0x04, 0x85, 0x11, 0x85, 0x13, 0x85, 0x15, 0x60, 0xa5, 0xfe, 0x85, 0x00, 0xa5, 0xfe,
            0x29, 0x03, 0x18, 0x69, 0x02, 0x85, 0x01, 0x60, 0x20, 0x4d, 0x06, 0x20, 0x8d, 0x06, 0x20, 0xc3,
            0x06, 0x20, 0x19, 0x07, 0x20, 0x20, 0x07, 0x20, 0x2d, 0x07, 0x4c, 0x38, 0x06, 0xa5, 0xff, 0xc9,
            0x77, 0xf0, 0x0d, 0xc9, 0x64, 0xf0, 0x14, 0xc9, 0x73, 0xf0, 0x1b, 0xc9, 0x61, 0xf0, 0x22, 0x60,
            0xa9, 0x04, 0x24, 0x02, 0xd0, 0x26, 0xa9, 0x01, 0x85, 0x02, 0x60, 0xa9, 0x08, 0x24, 0x02, 0xd0,
            0x1b, 0xa9, 0x02, 0x85, 0x02, 0x60, 0xa9, 0x01, 0x24, 0x02, 0xd0, 0x10, 0xa9, 0x04, 0x85, 0x02,
            0x60, 0xa9, 0x02, 0x24, 0x02, 0xd0, 0x05, 0xa9, 0x08, 0x85, 0x02, 0x60, 0x60, 0x20, 0x94, 0x06,
            0x20, 0xa8, 0x06, 0x60, 0xa5, 0x00, 0xc5, 0x10, 0xd0, 0x0d, 0xa5, 0x01, 0xc5, 0x11, 0xd0, 0x07,
            0xe6, 0x03, 0xe6, 0x03, 0x20, 0x2a, 0x06, 0x60, 0xa2, 0x02, 0xb5, 0x10, 0xc5, 0x10, 0xd0, 0x06,
            0xb5, 0x11, 0xc5, 0x11, 0xf0, 0x09, 0xe8, 0xe8, 0xe4, 0x03, 0xf0, 0x06, 0x4c, 0xaa, 0x06, 0x4c,
            0x35, 0x07, 0x60, 0xa6, 0x03, 0xca, 0x8a, 0xb5, 0x10, 0x95, 0x12, 0xca, 0x10, 0xf9, 0xa5, 0x02,
            0x4a, 0xb0, 0x09, 0x4a, 0xb0, 0x19, 0x4a, 0xb0, 0x1f, 0x4a, 0xb0, 0x2f, 0xa5, 0x10, 0x38, 0xe9,
            0x20, 0x85, 0x10, 0x90, 0x01, 0x60, 0xc6, 0x11, 0xa9, 0x01, 0xc5, 0x11, 0xf0, 0x28, 0x60, 0xe6,
            0x10, 0xa9, 0x1f, 0x24, 0x10, 0xf0, 0x1f, 0x60, 0xa5, 0x10, 0x18, 0x69, 0x20, 0x85, 0x10, 0xb0,
            0x01, 0x60, 0xe6, 0x11, 0xa9, 0x06, 0xc5, 0x11, 0xf0, 0x0c, 0x60, 0xc6, 0x10, 0xa5, 0x10, 0x29,
            0x1f, 0xc9, 0x1f, 0xf0, 0x01, 0x60, 0x4c, 0x35, 0x07, 0xa0, 0x00, 0xa5, 0xfe, 0x91, 0x00, 0x60,
            0xa6, 0x03, 0xa9, 0x00, 0x81, 0x10, 0xa2, 0x00, 0xa9, 0x01, 0x81, 0x10, 0x60, 0xa2, 0x00, 0xea,
            0xea, 0xca, 0xd0, 0xfb, 0x60
        ]);
    }

    #[test]
    fn test_end_to_end() {
        let mut cpu = CPU::new(fake_rom());
        cpu.program_counter = 0x8000;
        cpu.start();

        assert_eq!(cpu.x, 0xc1)
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

        let actual = cpu.bus.read_byte(0x04);
        assert_eq!(cpu.a, cpu.bus.read_byte(0x04));
    }
}
