use codegen::Scope;
use serde_json::{Result, Value};
use std::collections::HashMap;
use std::fs::{read_to_string, write};
use std::u8;

fn main() -> Result<()> {
    let file = read_to_string("ops_codes.json").unwrap();
    let value: Value = serde_json::from_str(&file)?;

    let mut scope = Scope::new();
    scope.import("super", "Instruction");
    scope.import("super", "InstructionType");
    scope.import("super", "MemoryAdressingMode");

    scope.raw("pub const INSTRUCTION_SET: [Instruction; 255] = [");

    let mut instructions = HashMap::new();
    for item in value.as_array().unwrap() {
        let mnemonic = item["name"].as_str().unwrap();

        let op_code = item["opcode"].as_str().unwrap().to_owned().replace("$", "");
        let op_code = u8::from_str_radix(&op_code, 16).unwrap();

        let bytes = item["bytes"].as_str().unwrap().to_owned();
        let cycles = item["cycles"].as_i64().unwrap();
        let plus_cycle = item["+1"].as_bool().unwrap();

        let instruction = format!("InstructionType::{}", mnemonic.to_uppercase());

        let memory_addressing = format!("{}", item["mode"].as_str().unwrap());
        let memory_addressing = format!("MemoryAdressingMode::{}", memory_addressing);

        let instruction = format!(
            "    instruction!(\"{}\", {:#x}, {}, {}, {}, {}, {}),",
            mnemonic, op_code, bytes, cycles, instruction, memory_addressing, plus_cycle
        );
        instructions.insert(op_code, instruction);
    }

    for i in 0..255 {
        let instruction = match instructions.get(&i) {
            Some(instruction) => instruction.to_owned(),
            None => format!("    instruction!(\"NotImplemented\", {:#x}, 0, 0, InstructionType::NotImplemented, MemoryAdressingMode::Immediate, false),", i),
        };
        scope.raw(&instruction);
    }

    scope.raw("];");

    // scope.raw("impl Into<u8> for InstructionType {");
    // scope.raw("fn into(self) -> u8 {");
    // scope.raw("match self {");
    // for item in value.as_array().unwrap() {
    //     let mnemonic = item["name"].as_str().unwrap();
    //     let op_code = item["opcode"].as_str().unwrap().to_owned().replace("$", "");
    //     let op_code = u8::from_str_radix(&op_code, 16).unwrap();

    //     let instruction = format!("InstructionType::{}", mnemonic.to_uppercase());
    //     let op_code = format!("{:#x}", op_code);
    //     let match_case = format!("{} => {},", instruction, op_code);
    //     scope.raw(&match_case);
    // }
    // scope.raw("}");
    // scope.raw("}");
    // scope.raw("}");

    write("src/cpu/instructions/instruction_set.rs", scope.to_string()).unwrap();
    std::process::Command::new("cargo")
        .arg("fmt")
        .output()
        .expect("Format failed");
    Ok(())
}
