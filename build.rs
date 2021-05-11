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

    scope.raw("pub (crate) const INSTRUCTION_SET: [Instruction; 255] = [");

    let mut instructions = HashMap::new();
    for item in value.as_array().unwrap() {
        let mnemonic = item["name"].as_str().unwrap();

        let op_code = item["opcode"].as_str().unwrap().to_owned().replace("$", "");
        let op_code = u8::from_str_radix(&op_code, 16).unwrap();

        let bytes = item["bytes"].as_str().unwrap().to_owned();

        let instruction = format!("InstructionType::{}", mnemonic.to_uppercase());

        let memory_addressing = format!("{}", item["mode"].as_str().unwrap());
        let memory_addressing = format!("MemoryAdressingMode::{}", memory_addressing);

        let instruction = format!(
            "    instruction!(\"{}\", {:#x}, {}, {}, {}, {}),",
            mnemonic, op_code, bytes, "0", instruction, memory_addressing
        );
        instructions.insert(op_code, instruction);
    }

    for i in 0..255 {
        let instruction = match instructions.get(&i) {
            Some(instruction) => instruction.to_owned(),
            None => format!("    instruction!(\"NotImplemented\", {:#x}, 0, 0, InstructionType::NotImplemented, MemoryAdressingMode::Immediate),", i),
        };
        scope.raw(&instruction);
    }

    scope.raw("];");

    write("src/cpu/instructions/instruction_set.rs", scope.to_string()).unwrap();
    std::process::Command::new("cargo")
        .arg("fmt")
        .output()
        .expect("Format failed");
    Ok(())
}
