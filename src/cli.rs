use std::io;

use crate::ast::Program;
use crate::bytecode::{Bytecode, BytecodeCompiler};
use crate::bytecode_artifact::{deserialize_bytecode, serialize_bytecode};
use crate::parser::Parser;
use crate::resolver::{ResolveResult, Resolver};
use crate::vm::Vm;
use crate::{Lexer, Token, TokenType, TOKEN_TYPE_NAMES};

const USAGE: &str = "Usage: lune [--no-tokens] <file.lune>\n       lune --emit-bytecode <output.lbc> <file.lune>\n       lune --run-bytecode <file.lbc>\n       lune --help";

#[derive(Debug, PartialEq, Eq)]
enum Command {
    Help,
    Run {
        source_path: String,
        print_tokens: bool,
    },
    EmitBytecode {
        source_path: String,
        output_path: String,
    },
    RunBytecode {
        bytecode_path: String,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub struct CliOutcome {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub trait CliFileSystem {
    fn read_to_string(&self, path: &str) -> io::Result<String>;
    fn write_string(&self, path: &str, content: &str) -> io::Result<()>;
}

pub fn run_cli(argv: &[String], file_system: &impl CliFileSystem) -> CliOutcome {
    match parse_args(argv) {
        Ok(Command::Help) => CliOutcome {
            exit_code: 0,
            stdout: format!("{USAGE}\n"),
            stderr: String::new(),
        },
        Ok(Command::Run {
            source_path,
            print_tokens,
        }) => run_source_file(file_system, &source_path, print_tokens),
        Ok(Command::EmitBytecode {
            source_path,
            output_path,
        }) => compile_source_file(file_system, &source_path, &output_path),
        Ok(Command::RunBytecode { bytecode_path }) => {
            run_bytecode_file(file_system, &bytecode_path)
        }
        Err(message) => CliOutcome {
            exit_code: 1,
            stdout: String::new(),
            stderr: format!("{message}\n{USAGE}\n"),
        },
    }
}

fn parse_args(argv: &[String]) -> Result<Command, &'static str> {
    if argv.is_empty() {
        return Err("error: missing input file");
    }

    if argv[0] == "-h" || argv[0] == "--help" {
        return Ok(Command::Help);
    }

    if argv[0] == "--emit-bytecode" {
        if argv.len() != 3 {
            return Err("error: expected --emit-bytecode <output.lbc> <file.lune>");
        }
        return Ok(Command::EmitBytecode {
            output_path: argv[1].clone(),
            source_path: argv[2].clone(),
        });
    }

    if argv[0] == "--run-bytecode" {
        if argv.len() != 2 {
            return Err("error: expected --run-bytecode <file.lbc>");
        }
        return Ok(Command::RunBytecode {
            bytecode_path: argv[1].clone(),
        });
    }

    let mut print_tokens = true;
    let mut source_path: Option<String> = None;

    for arg in argv {
        match arg.as_str() {
            "--no-tokens" => print_tokens = false,
            _ if arg.starts_with('-') => return Err("error: unknown option"),
            _ => {
                if source_path.is_some() {
                    return Err("error: expected a single input file");
                }
                source_path = Some(arg.clone());
            }
        }
    }

    let source_path = source_path.ok_or("error: missing input file")?;
    Ok(Command::Run {
        source_path,
        print_tokens,
    })
}

fn run_source_file(
    file_system: &impl CliFileSystem,
    source_path: &str,
    print_tokens: bool,
) -> CliOutcome {
    let source = match file_system.read_to_string(source_path) {
        Ok(content) => content,
        Err(error) => return read_error_outcome(error),
    };

    let mut stdout = String::new();
    let mut stderr = String::new();

    let mut lexer = Lexer::new(&source);
    let lexed = lexer.tokenize();
    if print_tokens {
        write_tokens(&mut stdout, &lexed.tokens);
    }

    append_lexer_errors(&mut stderr, &lexed.diagnostics);
    if !lexed.diagnostics.is_empty() {
        return CliOutcome {
            exit_code: 1,
            stdout,
            stderr,
        };
    }

    let parsed = Parser::new(&lexed.tokens).parse();
    append_parser_errors(&mut stderr, &parsed.diagnostics);
    if !parsed.diagnostics.is_empty() {
        return CliOutcome {
            exit_code: 1,
            stdout,
            stderr,
        };
    }

    let resolved = Resolver::new().resolve(&parsed.program);
    append_resolver_diagnostics(&mut stderr, &resolved.diagnostics);

    let bytecode = BytecodeCompiler::new().compile(&resolved.resolved_program);
    let vm_result = Vm::new().run(&bytecode);
    append_vm_errors(&mut stderr, &vm_result.diagnostics);
    append_line(&mut stdout, &format!("vm stack: {:?}", vm_result.stack));

    CliOutcome {
        exit_code: if vm_result.diagnostics.is_empty() {
            0
        } else {
            1
        },
        stdout,
        stderr,
    }
}

fn compile_source_file(
    file_system: &impl CliFileSystem,
    source_path: &str,
    output_path: &str,
) -> CliOutcome {
    let source = match file_system.read_to_string(source_path) {
        Ok(content) => content,
        Err(error) => return read_error_outcome(error),
    };

    let mut stderr = String::new();
    let compilation = compile_source(&source);

    append_lexer_errors(&mut stderr, &compilation.lexed.diagnostics);
    append_parser_errors(&mut stderr, &compilation.parsed.diagnostics);
    append_resolver_diagnostics(&mut stderr, &compilation.resolved.diagnostics);

    if !compilation.lexed.diagnostics.is_empty() || !compilation.parsed.diagnostics.is_empty() {
        return CliOutcome {
            exit_code: 1,
            stdout: String::new(),
            stderr,
        };
    }

    let encoded = serialize_bytecode(&compilation.bytecode);
    if let Err(error) = file_system.write_string(output_path, &encoded) {
        return CliOutcome {
            exit_code: 1,
            stdout: String::new(),
            stderr: format!("error: {error}\n"),
        };
    }

    CliOutcome {
        exit_code: 0,
        stdout: format!("wrote bytecode: {output_path}\n"),
        stderr,
    }
}

fn run_bytecode_file(file_system: &impl CliFileSystem, bytecode_path: &str) -> CliOutcome {
    let encoded = match file_system.read_to_string(bytecode_path) {
        Ok(content) => content,
        Err(error) => return read_error_outcome(error),
    };

    let bytecode = match deserialize_bytecode(&encoded) {
        Ok(decoded) => decoded,
        Err(error) => {
            return CliOutcome {
                exit_code: 1,
                stdout: String::new(),
                stderr: format!("bytecode error: {}\n", error.message()),
            };
        }
    };

    let vm_result = Vm::new().run(&bytecode);
    let mut stderr = String::new();
    append_vm_errors(&mut stderr, &vm_result.diagnostics);

    CliOutcome {
        exit_code: if vm_result.diagnostics.is_empty() {
            0
        } else {
            1
        },
        stdout: format!("vm stack: {:?}\n", vm_result.stack),
        stderr,
    }
}

struct SourceCompilation {
    lexed: crate::LexerResult,
    parsed: crate::parser::ParseResult,
    resolved: ResolveResult,
    bytecode: Bytecode,
}

fn compile_source(source: &str) -> SourceCompilation {
    let mut lexer = Lexer::new(source);
    let lexed = lexer.tokenize();

    if !lexed.diagnostics.is_empty() {
        return SourceCompilation {
            lexed,
            parsed: crate::parser::ParseResult {
                program: Program::new(Vec::new()),
                diagnostics: Vec::new(),
            },
            resolved: ResolveResult {
                resolved_program: Program::new(Vec::new()),
                diagnostics: Vec::new(),
            },
            bytecode: Bytecode {
                constants: Vec::new(),
                instructions: Vec::new(),
            },
        };
    }

    let parsed = Parser::new(&lexed.tokens).parse();
    if !parsed.diagnostics.is_empty() {
        return SourceCompilation {
            lexed,
            parsed,
            resolved: ResolveResult {
                resolved_program: Program::new(Vec::new()),
                diagnostics: Vec::new(),
            },
            bytecode: Bytecode {
                constants: Vec::new(),
                instructions: Vec::new(),
            },
        };
    }

    let resolved = Resolver::new().resolve(&parsed.program);
    let bytecode = BytecodeCompiler::new().compile(&resolved.resolved_program);
    SourceCompilation {
        lexed,
        parsed,
        resolved,
        bytecode,
    }
}

fn append_lexer_errors(stderr: &mut String, diagnostics: &[crate::Diagnostic]) {
    for diagnostic in diagnostics {
        append_line(
            stderr,
            &format!(
                "lexer error: {} at {}:{}",
                diagnostic.message(),
                diagnostic.line,
                diagnostic.column
            ),
        );
    }
}

fn append_parser_errors(stderr: &mut String, diagnostics: &[crate::parser::ParserDiagnostic]) {
    for diagnostic in diagnostics {
        append_line(
            stderr,
            &format!(
                "parser error: {} at {}:{}",
                diagnostic.kind.message(),
                diagnostic.line,
                diagnostic.column
            ),
        );
    }
}

fn append_resolver_diagnostics(
    stderr: &mut String,
    diagnostics: &[crate::resolver::ResolverDiagnostic],
) {
    for diagnostic in diagnostics {
        append_line(
            stderr,
            &format!(
                "resolver warning: {} ({})",
                diagnostic.kind.message(),
                diagnostic.name
            ),
        );
    }
}

fn append_vm_errors(stderr: &mut String, diagnostics: &[crate::vm::VmDiagnostic]) {
    for diagnostic in diagnostics {
        append_line(
            stderr,
            &format!(
                "vm error: {} at instruction {} (constant index {})",
                diagnostic.kind.message(),
                diagnostic.instruction_offset,
                diagnostic.constant_index
            ),
        );
    }
}

fn read_error_outcome(error: io::Error) -> CliOutcome {
    CliOutcome {
        exit_code: 1,
        stdout: String::new(),
        stderr: format!("error: {error}\n"),
    }
}

fn write_tokens(stdout: &mut String, tokens: &[Token]) {
    for token in tokens {
        append_line(
            stdout,
            &format!(
                "{}\t\"{}\"\t({}:{})",
                token_name(token.token_type),
                token.lexeme,
                token.line,
                token.column
            ),
        );
    }
}

fn append_line(output: &mut String, line: &str) {
    output.push_str(line);
    output.push('\n');
}

fn token_name(token_type: TokenType) -> &'static str {
    TOKEN_TYPE_NAMES
        .iter()
        .find_map(|(kind, name)| (*kind == token_type).then_some(*name))
        .unwrap_or("unknown")
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;

    use super::*;

    #[derive(Clone, Default)]
    struct InMemoryFileSystem {
        files: Rc<RefCell<HashMap<String, String>>>,
    }

    impl InMemoryFileSystem {
        fn with_file(path: &str, content: &str) -> Self {
            let fs = Self::default();
            fs.files
                .borrow_mut()
                .insert(path.to_string(), content.to_string());
            fs
        }

        fn read_written(&self, path: &str) -> Option<String> {
            self.files.borrow().get(path).cloned()
        }
    }

    impl CliFileSystem for InMemoryFileSystem {
        fn read_to_string(&self, path: &str) -> io::Result<String> {
            self.files
                .borrow()
                .get(path)
                .cloned()
                .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))
        }

        fn write_string(&self, path: &str, content: &str) -> io::Result<()> {
            self.files
                .borrow_mut()
                .insert(path.to_string(), content.to_string());
            Ok(())
        }
    }

    fn str_args(args: &[&str]) -> Vec<String> {
        args.iter().map(|arg| arg.to_string()).collect()
    }

    #[test]
    fn parses_emit_bytecode_command() {
        let args = str_args(&["--emit-bytecode", "out.lbc", "demo.lune"]);
        assert_eq!(
            parse_args(&args),
            Ok(Command::EmitBytecode {
                source_path: "demo.lune".to_string(),
                output_path: "out.lbc".to_string()
            })
        );
    }

    #[test]
    fn parses_run_bytecode_command() {
        let args = str_args(&["--run-bytecode", "program.lbc"]);
        assert_eq!(
            parse_args(&args),
            Ok(Command::RunBytecode {
                bytecode_path: "program.lbc".to_string()
            })
        );
    }

    #[test]
    fn emits_bytecode_and_runs_it() {
        let fs = InMemoryFileSystem::with_file("demo.lune", "1 \"x\"");

        let compile = run_cli(
            &str_args(&["--emit-bytecode", "demo.lbc", "demo.lune"]),
            &fs,
        );
        assert_eq!(compile.exit_code, 0);
        assert!(compile.stderr.is_empty());
        assert!(fs.read_written("demo.lbc").is_some());

        let run = run_cli(&str_args(&["--run-bytecode", "demo.lbc"]), &fs);
        assert_eq!(run.exit_code, 0);
        assert!(run
            .stdout
            .contains("vm stack: [Number(1.0), String(\"x\")]"));
    }

    #[test]
    fn reports_invalid_bytecode_artifact() {
        let fs = InMemoryFileSystem::with_file("broken.lbc", "bad");
        let result = run_cli(&str_args(&["--run-bytecode", "broken.lbc"]), &fs);

        assert_eq!(result.exit_code, 1);
        assert!(result.stderr.contains("bytecode error:"));
    }
}
