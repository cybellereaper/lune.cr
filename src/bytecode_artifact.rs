use crate::bytecode::{Bytecode, Instruction, Value};

const MAGIC_HEADER: &str = "LUNEBC1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BytecodeArtifactError {
    InvalidHeader,
    UnexpectedEof,
    InvalidConstantTag(String),
    InvalidInstructionTag(String),
    InvalidNumber(String),
    InvalidBoolean(String),
    InvalidInstructionIndex(String),
}

impl BytecodeArtifactError {
    pub fn message(&self) -> String {
        match self {
            BytecodeArtifactError::InvalidHeader => "invalid bytecode header".to_string(),
            BytecodeArtifactError::UnexpectedEof => {
                "unexpected end of bytecode artifact".to_string()
            }
            BytecodeArtifactError::InvalidConstantTag(tag) => {
                format!("invalid constant tag: {tag}")
            }
            BytecodeArtifactError::InvalidInstructionTag(tag) => {
                format!("invalid instruction tag: {tag}")
            }
            BytecodeArtifactError::InvalidNumber(value) => {
                format!("invalid numeric value: {value}")
            }
            BytecodeArtifactError::InvalidBoolean(value) => {
                format!("invalid boolean value: {value}")
            }
            BytecodeArtifactError::InvalidInstructionIndex(value) => {
                format!("invalid instruction index: {value}")
            }
        }
    }
}

pub fn serialize_bytecode(bytecode: &Bytecode) -> String {
    let mut lines = Vec::new();
    lines.push(MAGIC_HEADER.to_string());
    lines.push(format!("C {}", bytecode.constants.len()));

    for value in &bytecode.constants {
        match value {
            Value::Number(n) => lines.push(format!("N {n}")),
            Value::String(s) => lines.push(format!("S {}", escape_string(s))),
            Value::Bool(true) => lines.push("B 1".to_string()),
            Value::Bool(false) => lines.push("B 0".to_string()),
            Value::Null => lines.push("Z".to_string()),
        }
    }

    lines.push(format!("I {}", bytecode.instructions.len()));
    for instruction in &bytecode.instructions {
        match instruction {
            Instruction::PushConst(index) => lines.push(format!("P {index}")),
            Instruction::Halt => lines.push("H".to_string()),
        }
    }

    lines.join("\n")
}

pub fn deserialize_bytecode(encoded: &str) -> Result<Bytecode, BytecodeArtifactError> {
    let mut lines = encoded.lines();

    let header = next_line(&mut lines)?;
    if header != MAGIC_HEADER {
        return Err(BytecodeArtifactError::InvalidHeader);
    }

    let constants_meta = next_line(&mut lines)?;
    let constant_count = parse_count(constants_meta, "C")?;
    let mut constants = Vec::with_capacity(constant_count);
    for _ in 0..constant_count {
        let line = next_line(&mut lines)?;
        constants.push(parse_constant(line)?);
    }

    let instructions_meta = next_line(&mut lines)?;
    let instruction_count = parse_count(instructions_meta, "I")?;
    let mut instructions = Vec::with_capacity(instruction_count);
    for _ in 0..instruction_count {
        let line = next_line(&mut lines)?;
        instructions.push(parse_instruction(line)?);
    }

    Ok(Bytecode {
        constants,
        instructions,
    })
}

fn next_line<'a>(
    lines: &mut impl Iterator<Item = &'a str>,
) -> Result<&'a str, BytecodeArtifactError> {
    lines.next().ok_or(BytecodeArtifactError::UnexpectedEof)
}

fn parse_count(line: &str, expected_tag: &str) -> Result<usize, BytecodeArtifactError> {
    let (tag, count_text) = split_tag_and_payload(line)?;
    if tag != expected_tag {
        return if expected_tag == "C" {
            Err(BytecodeArtifactError::InvalidConstantTag(tag.to_string()))
        } else {
            Err(BytecodeArtifactError::InvalidInstructionTag(
                tag.to_string(),
            ))
        };
    }

    count_text
        .parse::<usize>()
        .map_err(|_| BytecodeArtifactError::InvalidNumber(count_text.to_string()))
}

fn parse_constant(line: &str) -> Result<Value, BytecodeArtifactError> {
    if line == "Z" {
        return Ok(Value::Null);
    }

    let (tag, payload) = split_tag_and_payload(line)?;
    match tag {
        "N" => payload
            .parse::<f64>()
            .map(Value::Number)
            .map_err(|_| BytecodeArtifactError::InvalidNumber(payload.to_string())),
        "S" => Ok(Value::String(unescape_string(payload))),
        "B" => match payload {
            "1" => Ok(Value::Bool(true)),
            "0" => Ok(Value::Bool(false)),
            _ => Err(BytecodeArtifactError::InvalidBoolean(payload.to_string())),
        },
        _ => Err(BytecodeArtifactError::InvalidConstantTag(tag.to_string())),
    }
}

fn parse_instruction(line: &str) -> Result<Instruction, BytecodeArtifactError> {
    if line == "H" {
        return Ok(Instruction::Halt);
    }

    let (tag, payload) = split_tag_and_payload(line)?;
    match tag {
        "P" => payload
            .parse::<usize>()
            .map(Instruction::PushConst)
            .map_err(|_| BytecodeArtifactError::InvalidInstructionIndex(payload.to_string())),
        _ => Err(BytecodeArtifactError::InvalidInstructionTag(
            tag.to_string(),
        )),
    }
}

fn split_tag_and_payload(line: &str) -> Result<(&str, &str), BytecodeArtifactError> {
    line.split_once(' ')
        .ok_or(BytecodeArtifactError::UnexpectedEof)
}

fn escape_string(value: &str) -> String {
    value
        .chars()
        .flat_map(|c| match c {
            '\\' => ['\\', '\\'].into_iter().collect::<Vec<char>>(),
            '\n' => ['\\', 'n'].into_iter().collect::<Vec<char>>(),
            '\t' => ['\\', 't'].into_iter().collect::<Vec<char>>(),
            _ => [c].into_iter().collect::<Vec<char>>(),
        })
        .collect()
}

fn unescape_string(value: &str) -> String {
    let mut chars = value.chars();
    let mut output = String::new();

    while let Some(ch) = chars.next() {
        if ch != '\\' {
            output.push(ch);
            continue;
        }

        match chars.next() {
            Some('n') => output.push('\n'),
            Some('t') => output.push('\t'),
            Some('\\') => output.push('\\'),
            Some(other) => {
                output.push('\\');
                output.push(other);
            }
            None => output.push('\\'),
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_bytecode_with_all_constant_types() {
        let bytecode = Bytecode {
            constants: vec![
                Value::Number(42.0),
                Value::String("line1\nline2\t\\ok".to_string()),
                Value::Bool(true),
                Value::Bool(false),
                Value::Null,
            ],
            instructions: vec![
                Instruction::PushConst(0),
                Instruction::PushConst(1),
                Instruction::PushConst(2),
                Instruction::Halt,
            ],
        };

        let encoded = serialize_bytecode(&bytecode);
        let decoded = deserialize_bytecode(&encoded).expect("valid artifact");
        assert_eq!(decoded, bytecode);
    }

    #[test]
    fn rejects_invalid_header() {
        let encoded = "NOT_LUNE\nC 0\nI 1\nH";
        let error = deserialize_bytecode(encoded).expect_err("must reject invalid header");
        assert_eq!(error, BytecodeArtifactError::InvalidHeader);
    }
}
