use std::{
    collections::VecDeque,
    fs::File,
    io::{BufRead, BufReader},
    ops::Deref,
};

use nes::{
    bus::MemoryBus,
    cpu::{
        instructions::{get_instruction_from_opcode, Instruction},
        processor_status::ProcessorStatus,
        CPU,
    },
    rom::Rom,
};

#[derive(Default)]
struct CPURecorder {
    count: usize,
    expected_cpu_state: VecDeque<CPU>,
    expected_instruction: VecDeque<Instruction>,
}

impl CPURecorder {
    fn new_from_nes_log(path: &str) -> Self {
        let file = File::open(path).expect("nes log not found");
        let reader = BufReader::new(file);
        let mut expected_instruction = VecDeque::new();
        let mut expected_cpu_state = VecDeque::new();
        for line in reader.lines() {
            let line = line.expect("line not found");
            let parts: Vec<&str> = line.split("A:").collect();
            let instruction_parts: Vec<&str> = line.split_whitespace().collect();

            // Extract address (not used directly in the Instruction struct)
            let _address = instruction_parts[0];

            // Extract opcode and operands
            let opcode_str = instruction_parts[1];

            expected_instruction.push_back(
                get_instruction_from_opcode(
                    u8::from_str_radix(opcode_str, 16).expect("could not convert str to u8")
                        as usize,
                )
                .deref()
                .clone(),
            );

            let state_parts: Vec<&str> = parts[1].split_whitespace().collect();

            let program_counter = u16::from_str_radix(instruction_parts[0], 16).unwrap();
            let a = u8::from_str_radix(&state_parts[0], 16).unwrap();
            let x = u8::from_str_radix(&state_parts[1][2..4], 16).unwrap();
            let y = u8::from_str_radix(&state_parts[2][2..4], 16).unwrap();
            let processor_status = ProcessorStatus::from_bits_truncate(
                u8::from_str_radix(&state_parts[3][2..4], 16).unwrap(),
            );
            let stack_pointer = u8::from_str_radix(&state_parts[4][3..5], 16).unwrap();
            let cycle = u64::from_str_radix(&state_parts.last().unwrap()[4..], 10).unwrap();

            expected_cpu_state.push_back(CPU {
                program_counter,
                a,
                x,
                y,
                processor_status,
                stack_pointer,
                bus: test_rom(),
                cycle,
            })
        }
        // State has to be one ahead
        expected_cpu_state.pop_front().unwrap();

        CPURecorder {
            count: 0,
            expected_cpu_state,
            expected_instruction,
        }
    }

    fn check_state(&mut self, cpu: &CPU) {
        let expected = self
            .expected_cpu_state
            .pop_front()
            .expect("no call where expected");
        assert_eq!(
            cpu.a, expected.a,
            "A Register Not as expected as {:x}",
            cpu.program_counter
        );
        assert_eq!(cpu.x, expected.x, "X Register Not as expected");
        assert_eq!(cpu.y, expected.y, "Y Register Not as expected");
        assert_eq!(
            cpu.program_counter, expected.program_counter,
            "PC Not as expected:\nact: {:x}\nexp: {:x}",
            cpu.program_counter, expected.program_counter
        );
        assert_eq!(
            cpu.stack_pointer, expected.stack_pointer,
            "SP Not as expected"
        );
        assert_eq!(
            cpu.processor_status, expected.processor_status,
            "CPU Processor State not as expected at {:x}\nact: {}\nexp: {}",
            expected.program_counter, cpu.processor_status, expected.processor_status
        );

        assert_eq!(
            cpu.cycle, expected.cycle,
            "Cycle count not as expected\nexp: {} act:{}",
            expected.cycle, cpu.cycle
        );

        self.count += 1;
    }

    fn check_instruction(&mut self, cpu: &CPU, instruction: &Instruction) {
        let expected_instruction = self
            .expected_instruction
            .pop_front()
            .expect("no instruction where expected");
        assert_eq!(
            *instruction, expected_instruction,
            "Bad instruction at {:x}\nexp: {:?}\nact: {:?}",
            cpu.program_counter, expected_instruction, instruction,
        );
    }
}

fn test_rom() -> MemoryBus {
    let test_rom =
        Rom::from_path("./tests/nestest.nes".to_owned()).expect("could not open the rom");
    let bus = MemoryBus::new(test_rom);
    bus
}

#[test]
fn test_cpu() {
    let bus = test_rom();
    let mut cpu = CPU::new_with_state(
        bus.clone(),
        0xC000,
        0xFD,
        0,
        0,
        0,
        ProcessorStatus::from_bits_truncate(0x24),
        7,
    );

    let mut cpu_recorder = CPURecorder::new_from_nes_log("./tests/nestest.log");

    cpu.start_with_callback(|cpu, instruction| {
        cpu_recorder.check_state(&cpu);
        cpu_recorder.check_instruction(&cpu, instruction);
    });
}
