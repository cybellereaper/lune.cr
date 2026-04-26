use crate::bytecode::{Bytecode, Instruction, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmDiagnosticKind {
    InvalidConstantIndex,
}

impl VmDiagnosticKind {
    pub fn message(self) -> &'static str {
        match self {
            VmDiagnosticKind::InvalidConstantIndex => {
                "Instruction referenced a constant index that does not exist"
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VmDiagnostic {
    pub kind: VmDiagnosticKind,
    pub instruction_offset: usize,
    pub constant_index: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VmResult {
    pub stack: Vec<Value>,
    pub diagnostics: Vec<VmDiagnostic>,
}

#[derive(Default)]
pub struct Vm;

impl Vm {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&self, bytecode: &Bytecode) -> VmResult {
        let mut stack = Vec::new();
        let mut diagnostics = Vec::new();

        for (instruction_offset, instruction) in bytecode.instructions.iter().enumerate() {
            match instruction {
                Instruction::PushConst(idx) => {
                    if let Some(value) = bytecode.constants.get(*idx) {
                        stack.push(value.clone());
                    } else {
                        diagnostics.push(VmDiagnostic {
                            kind: VmDiagnosticKind::InvalidConstantIndex,
                            instruction_offset,
                            constant_index: *idx,
                        });
                    }
                }
                Instruction::Halt => break,
            }
        }

        VmResult { stack, diagnostics }
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
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn reports_invalid_constant_index_without_panicking() {
        let bytecode = Bytecode {
            constants: vec![],
            instructions: vec![Instruction::PushConst(1), Instruction::Halt],
        };

        let result = Vm::new().run(&bytecode);

        assert!(result.stack.is_empty());
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(
            result.diagnostics[0].kind,
            VmDiagnosticKind::InvalidConstantIndex
        );
        assert_eq!(result.diagnostics[0].instruction_offset, 0);
        assert_eq!(result.diagnostics[0].constant_index, 1);
    }
}
