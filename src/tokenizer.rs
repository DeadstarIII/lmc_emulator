#![allow(unused)]

use std::collections::HashMap;
use std::fs::{self, *};
use std::io::{self, *};
use std::path::Path;

pub fn generate_tokenized_instructions(source_path: &Path) -> Result<Vec<i64>> {
    let source_file = File::open(source_path).unwrap();
    let buffer = BufReader::new(source_file);
    let mut preliminary_buffer = BufReader::new(File::open(source_path).unwrap());

    let mut tokenized_instructions: Vec<i64> = Vec::new();
    let mut line_number = 0;

    let mut labels: HashMap<String, HashMap<usize, i64>> = HashMap::new();
    let mut labels_wrapper: HashMap<String, usize> = HashMap::new();

    let is_label = |s: &str| {
        ![
            "ADD", "SUB", "STA", "LDA", "BRA", "BRZ", "BRP", "INP", "OUT", "HLT", "DAT",
        ]
        .contains(&s)
    };

    let mut memory_location = 0;
    for line in preliminary_buffer.lines() {
        let mut line = line.unwrap();

        if let Some(index) = line.find("//") {
            line.truncate(index);
        }

        if line.trim().is_empty() {
            continue;
        }

        let mut parts = line.trim().split_whitespace();

        if let Some(first) = parts.next() {
            if is_label(first) {
                labels_wrapper.insert(first.to_string(), memory_location);
            }
        }

        memory_location += 1;
    }

    let resolve_or_parse = |s: &str| -> i64 {
        labels_wrapper
            .get(s)
            .copied()
            .map(|n| n as i64)
            .unwrap_or_else(|| s.parse().unwrap())
    };

    for line in buffer.lines() {
        let mut line = line.unwrap();

        if let Some(index) = line.find("//") {
            line.truncate(index);
        }

        if line.trim().is_empty() {
            continue;
        }

        let instructions_vector: Vec<&str> = line.trim().split_whitespace().collect();
        let mut idx = 0;

        if is_label(instructions_vector[0]) {
            idx = 1;
        }

        match instructions_vector[idx].to_uppercase().as_str() {
            "ADD" => {
                tokenized_instructions.push(100 + resolve_or_parse(instructions_vector[idx + 1]))
            }
            "SUB" => {
                tokenized_instructions.push(200 + resolve_or_parse(instructions_vector[idx + 1]))
            }
            "STA" => {
                tokenized_instructions.push(300 + resolve_or_parse(instructions_vector[idx + 1]))
            }
            "LDA" => {
                let resolved_value = resolve_or_parse(instructions_vector[idx + 1]);
                tokenized_instructions.push(500 + resolved_value);
            }
            "BRA" => {
                tokenized_instructions.push(600 + resolve_or_parse(instructions_vector[idx + 1]))
            }
            "BRZ" => {
                tokenized_instructions.push(700 + resolve_or_parse(instructions_vector[idx + 1]))
            }
            "BRP" => {
                tokenized_instructions.push(800 + resolve_or_parse(instructions_vector[idx + 1]))
            }
            "INP" => tokenized_instructions.push(901),
            "OUT" => tokenized_instructions.push(902),
            "HLT" => tokenized_instructions.push(000),
            "DAT" => {
                if instructions_vector.len() > idx + 1 {
                    tokenized_instructions
                        .push(000 + instructions_vector[idx + 1].parse::<i64>().unwrap());
                } else {
                    tokenized_instructions.push(000);
                }
            }
            _ => {
                if let Some(instruction) = instructions_vector.get(1) {
                    let tokenized_instruction = match instruction.to_uppercase().as_str() {
                        "ADD" => {
                            100 + instructions_vector
                                .get(2)
                                .and_then(|&s| s.parse::<i64>().ok())
                                .unwrap_or(0)
                        }
                        "SUB" => {
                            200 + instructions_vector
                                .get(2)
                                .and_then(|&s| s.parse::<i64>().ok())
                                .unwrap_or(0)
                        }
                        "STA" => {
                            300 + instructions_vector
                                .get(2)
                                .and_then(|&s| s.parse::<i64>().ok())
                                .unwrap_or(0)
                        }
                        "LDA" => {
                            match instructions_vector
                                .get(1)
                                .and_then(|&s| s.parse::<i64>().ok())
                            {
                                Some(value) => 500 + value,
                                None => {
                                    eprintln!(
                                        "Error: LDA instruction missing operand on line {}",
                                        line_number + 1
                                    );
                                    continue;
                                }
                            }
                        }
                        "BRA" => {
                            600 + instructions_vector
                                .get(2)
                                .and_then(|&s| s.parse::<i64>().ok())
                                .unwrap_or(0)
                        }
                        "BRZ" => {
                            700 + instructions_vector
                                .get(2)
                                .and_then(|&s| s.parse::<i64>().ok())
                                .unwrap_or(0)
                        }
                        "BRP" => {
                            800 + instructions_vector
                                .get(2)
                                .and_then(|&s| s.parse::<i64>().ok())
                                .unwrap_or(0)
                        }
                        "INP" => 901,
                        "OUT" => 902,
                        "HLT" => 000,
                        "DAT" => {
                            if let Some(data) = instructions_vector.get(2) {
                                data.parse::<i64>().unwrap()
                            } else {
                                000
                            }
                        }
                        _ => {
                            panic!("Unknown Instruction: {}", line);
                            continue;
                        }
                    };
                    tokenized_instructions.push(tokenized_instruction);
                    let mut inner: HashMap<usize, i64> = HashMap::new();
                    inner.insert(line_number, tokenized_instruction);
                    if let Some(label) = line.split_whitespace().next() {
                        labels.insert(label.to_string(), inner);
                    }
                }
            }
        }
        line_number += 1;
    }

    Ok(tokenized_instructions)
}

pub fn generate_binary(tokenized_instructions: Vec<i64>, binary_path: &Path) -> Result<()> {
    let mut content = String::new();
    for tokenized_instruction in tokenized_instructions {
        content += &format!("{:03}", tokenized_instruction);
    }
    let mut binary = File::create(binary_path).unwrap();
    binary.write_all(content.as_bytes());
    Ok(())
}

pub fn generate_instructions(binary_path: &Path) -> Result<Vec<i64>> {
    let binary_file = File::open(binary_path).unwrap();
    let mut buffer = BufReader::new(binary_file);
    let mut content = String::new();
    buffer.read_to_string(&mut content);
    let mut tokenized_instructions: Vec<i64> = Vec::new();
    for chunk in content.as_bytes().chunks(3) {
        let opcode_str = std::str::from_utf8(chunk).unwrap();
        let opcode = opcode_str.parse::<i64>().unwrap();
        tokenized_instructions.push(opcode);
    }
    Ok(tokenized_instructions)
}
