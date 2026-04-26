use crate::bytecode::{Bytecode, Instruction, Value};

#[derive(Debug, Clone, PartialEq)]
pub struct VmResult {
    pub stack: Vec<Value>,
}

#[derive(Default)]
pub struct Vm;

impl Vm {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&self, bytecode: &Bytecode) -> VmResult {
        let mut stack = Vec::new();

        for instruction in &bytecode.instructions {
            match instruction {
                Instruction::PushConst(idx) => {
                    if let Some(value) = bytecode.constants.get(*idx) {
                        stack.push(value.clone());
                    }
                }
                Instruction::Halt => break,
            }
        }

        VmResult { stack }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn executes_push_const_instructions_and_stops_on_halt() {
        let bytecode = Bytecode {
            constants: vec![Value::Number(7.0)],
            instructions: vec![Instruction::PushConst(0), Instruction::Halt],
        };

        let result = Vm::new().run(&bytecode);

        assert_eq!(result.stack, vec![Value::Number(7.0)]);
    }
}
