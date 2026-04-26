use crate::ast::{AstNode, Program};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Null,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    PushConst(usize),
    Halt,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Bytecode {
    pub constants: Vec<Value>,
    pub instructions: Vec<Instruction>,
}

#[derive(Default)]
pub struct BytecodeCompiler;

impl BytecodeCompiler {
    pub fn new() -> Self {
        Self
    }

    pub fn compile(&self, program: &Program) -> Bytecode {
        let mut constants = Vec::new();
        let mut instructions = Vec::new();

        for node in &program.nodes {
            if let Some(value) = compile_value(node) {
                let index = constants.len();
                constants.push(value);
                instructions.push(Instruction::PushConst(index));
            }
        }

        instructions.push(Instruction::Halt);

        Bytecode {
            constants,
            instructions,
        }
    }
}

fn compile_value(node: &AstNode) -> Option<Value> {
    match node {
        AstNode::Number(n) => Some(Value::Number(*n)),
        AstNode::String(s) => Some(Value::String(s.clone())),
        AstNode::Bool(b) => Some(Value::Bool(*b)),
        AstNode::Null => Some(Value::Null),
        AstNode::Identifier(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiles_literals_into_constant_pushes_and_halt() {
        let program = Program::new(vec![AstNode::Number(2.0), AstNode::Null]);

        let bytecode = BytecodeCompiler::new().compile(&program);

        assert_eq!(bytecode.constants, vec![Value::Number(2.0), Value::Null]);
        assert_eq!(
            bytecode.instructions,
            vec![
                Instruction::PushConst(0),
                Instruction::PushConst(1),
                Instruction::Halt
            ]
        );
    }
}
