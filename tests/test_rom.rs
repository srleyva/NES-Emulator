use std::{
    collections::{vec_deque, VecDeque},
    fs::{self, File},
    io::{BufRead, BufReader},
    time::Duration,
};

use nes::{
    bus::MemoryBus,
    cpu::{
        instructions::{
            instruction_set::{self, INSTRUCTION_SET},
            Instruction,
        },
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
        for line in reader.lines() {
            let line = line.expect("line not found");
            let parts: Vec<&str> = line.split_whitespace().collect();

            // Extract address (not used directly in the Instruction struct)
            let _address = parts[0];

            // Extract opcode and operands
            let opcode_str = parts[1];

            println!("Addr: {} OpCode: {}", _address, opcode_str);

            expected_instruction.push_back(
                INSTRUCTION_SET[u8::from_str_radix(opcode_str, 16)
                    .expect("could not convert str to u8")
                    as usize],
            );
        }

        CPURecorder {
            count: 0,
            expected_cpu_state: VecDeque::new(),
            expected_instruction,
        }
    }

    fn check_state(&mut self, cpu: &CPU) {
        let expected = self
            .expected_cpu_state
            .pop_front()
            .expect("no call where expected");
        assert_eq!(
            *cpu, expected,
            "CPU State not as expected\nact: {:?}\nexp: {:?}",
            cpu, expected
        );
        self.count += 1;
    }

    fn check_instruction(&mut self, instruction: &Instruction) {
        assert_eq!(
            *instruction,
            self.expected_instruction
                .pop_front()
                .expect("no instruction where expected")
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
    let mut bus = test_rom();
    bus.write_byte(100, 0xa2);
    bus.write_byte(101, 0x01);
    bus.write_byte(102, 0xca);
    bus.write_byte(103, 0x88);
    bus.write_byte(104, 0x00);

    let mut cpu = CPU::new_with_state(
        bus.clone(),
        0xC000,
        0xFD,
        1,
        2,
        3,
        ProcessorStatus::default(),
    );

    let mut cpu_recorder = CPURecorder::new_from_nes_log("./tests/nestest.log");

    cpu.start_with_callback(|cpu, instruction| {
        // cpu_recorder.check_state(&cpu);
        cpu_recorder.check_instruction(instruction);
    });
}

// #[test]
// fn test_format_mem_access() {
//     let mut bus = test_rom();
//     // ORA ($33), Y
//     bus.write_byte(100, 0x11);
//     bus.write_byte(101, 0x33);

//     //data
//     bus.write_byte(0x33, 00);
//     bus.write_byte(0x34, 04);

//     //target cell
//     bus.write_byte(0x400, 0xAA);

//     let mut cpu = CPU::new_with_state(bus.clone(), 0x64, 0xFd, 0, 0, 0, ProcessorStatus::default());
//     let mut cpu_recorder = CPURecorder {
//         count: 0,
//         expected_cpu_state: VecDeque::from([
//             CPU::new_with_state(
//                 bus.clone(),
//                 0x66,
//                 0xFD,
//                 0xAA,
//                 0x00,
//                 0x00,
//                 ProcessorStatus::new(false, false, false, false, false, false, false, true),
//             ),
//             CPU::new_with_state(
//                 bus.clone(),
//                 0x67,
//                 0xFD,
//                 0xAA,
//                 0x00,
//                 0x00,
//                 ProcessorStatus::new(false, false, false, false, true, false, false, true),
//             ),
//         ]),
//         expected_instruction: VecDeque::from([
//             instruction_set::INSTRUCTION_SET[0x11].clone(),
//             instruction_set::INSTRUCTION_SET[0x00].clone(),
//         ]),
//     };

//     cpu.start_with_callback(|cpu, instruction| {
//         cpu_recorder.check_state(&cpu);
//         cpu_recorder.check_instruction(instruction);
//     });
// }
