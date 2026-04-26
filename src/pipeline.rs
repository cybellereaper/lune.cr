use crate::ast::Program;
use crate::bytecode::{Bytecode, BytecodeCompiler};
use crate::parser::{ParseResult, Parser};
use crate::resolver::{ResolveResult, Resolver};
use crate::vm::{Vm, VmResult};
use crate::{Lexer, LexerResult};

#[derive(Debug, Clone, PartialEq)]
pub struct PipelineOutput {
    pub lexed: LexerResult,
    pub parsed: ParseResult,
    pub resolved: ResolveResult,
    pub bytecode: Bytecode,
    pub vm_result: VmResult,
}

pub fn run_pipeline(source: &str) -> PipelineOutput {
    let mut lexer = Lexer::new(source);
    let lexed = lexer.tokenize();

    let parsed = parse_stage(&lexed);
    let can_execute = lexed.diagnostics.is_empty() && parsed.diagnostics.is_empty();

    let (resolved, bytecode, vm_result) = if can_execute {
        let resolved = Resolver::new().resolve(&parsed.program);
        let bytecode = BytecodeCompiler::new().compile(&resolved.resolved_program);
        let vm_result = Vm::new().run(&bytecode);
        (resolved, bytecode, vm_result)
    } else {
        (
            ResolveResult {
                resolved_program: Program::new(Vec::new()),
                diagnostics: Vec::new(),
            },
            Bytecode {
                constants: Vec::new(),
                instructions: Vec::new(),
            },
            VmResult {
                stack: Vec::new(),
                diagnostics: Vec::new(),
            },
        )
    };

    PipelineOutput {
        lexed,
        parsed,
        resolved,
        bytecode,
        vm_result,
    }
}

fn parse_stage(lexed: &LexerResult) -> ParseResult {
    if !lexed.diagnostics.is_empty() {
        return ParseResult {
            program: Program::new(Vec::new()),
            diagnostics: Vec::new(),
        };
    }

    Parser::new(&lexed.tokens).parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::Value;

    #[test]
    fn runs_all_stages_and_produces_vm_stack_values() {
        let output = run_pipeline("1 \"x\" true");

        assert!(output.lexed.diagnostics.is_empty());
        assert!(output.parsed.diagnostics.is_empty());
        assert_eq!(
            output.vm_result.stack,
            vec![
                Value::Number(1.0),
                Value::String("x".to_owned()),
                Value::Bool(true)
            ]
        );
        assert!(output.vm_result.diagnostics.is_empty());
    }

    #[test]
    fn short_circuits_parse_program_on_lexer_errors() {
        let output = run_pipeline("!");

        assert!(!output.lexed.diagnostics.is_empty());
        assert!(output.parsed.program.is_empty());
        assert!(output.vm_result.stack.is_empty());
    }
}
