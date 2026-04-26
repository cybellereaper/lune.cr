use std::io;

use crate::pipeline::run_pipeline;
use crate::{Token, TokenType, TOKEN_TYPE_NAMES};

const USAGE: &str = "Usage: lune [--no-tokens] <file.lune>\n       lune --help";

#[derive(Debug, PartialEq, Eq)]
enum Command {
    Help,
    Run {
        source_path: String,
        print_tokens: bool,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub struct CliOutcome {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub fn run_cli(argv: &[String], read_file: impl Fn(&str) -> io::Result<String>) -> CliOutcome {
    match parse_args(argv) {
        Ok(Command::Help) => CliOutcome {
            exit_code: 0,
            stdout: format!("{USAGE}\n"),
            stderr: String::new(),
        },
        Ok(Command::Run {
            source_path,
            print_tokens,
        }) => run_file(&source_path, print_tokens, read_file),
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

    let mut print_tokens = true;
    let mut source_path: Option<String> = None;

    for arg in argv {
        match arg.as_str() {
            "-h" | "--help" => return Ok(Command::Help),
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

fn run_file(
    source_path: &str,
    print_tokens: bool,
    read_file: impl Fn(&str) -> io::Result<String>,
) -> CliOutcome {
    let source = match read_file(source_path) {
        Ok(content) => content,
        Err(error) => {
            return CliOutcome {
                exit_code: 1,
                stdout: String::new(),
                stderr: format!("error: {error}\n"),
            };
        }
    };

    let output = run_pipeline(&source);
    let mut stdout = String::new();
    let mut stderr = String::new();

    if print_tokens {
        write_tokens(&mut stdout, &output.lexed.tokens);
    }

    for diagnostic in &output.lexed.diagnostics {
        append_line(
            &mut stderr,
            &format!(
                "lexer error: {} at {}:{}",
                diagnostic.message(),
                diagnostic.line,
                diagnostic.column
            ),
        );
    }

    for diagnostic in &output.parsed.diagnostics {
        append_line(
            &mut stderr,
            &format!(
                "parser error: {} at {}:{}",
                diagnostic.kind.message(),
                diagnostic.line,
                diagnostic.column
            ),
        );
    }

    for diagnostic in &output.resolved.diagnostics {
        append_line(
            &mut stderr,
            &format!(
                "resolver warning: {} ({})",
                diagnostic.kind.message(),
                diagnostic.name
            ),
        );
    }

    for diagnostic in &output.vm_result.diagnostics {
        append_line(
            &mut stderr,
            &format!(
                "vm error: {} at instruction {} (constant index {})",
                diagnostic.kind.message(),
                diagnostic.instruction_offset,
                diagnostic.constant_index
            ),
        );
    }

    if output.lexed.diagnostics.is_empty() {
        append_line(
            &mut stdout,
            &format!("vm stack: {:?}", output.vm_result.stack),
        );
    }

    let success = output.lexed.diagnostics.is_empty()
        && output.parsed.diagnostics.is_empty()
        && output.vm_result.diagnostics.is_empty();

    CliOutcome {
        exit_code: if success { 0 } else { 1 },
        stdout,
        stderr,
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
    use super::*;

    fn str_args(args: &[&str]) -> Vec<String> {
        args.iter().map(|arg| arg.to_string()).collect()
    }

    #[test]
    fn parses_help_option() {
        let args = str_args(&["--help"]);
        assert_eq!(parse_args(&args), Ok(Command::Help));
    }

    #[test]
    fn parses_no_tokens_option() {
        let args = str_args(&["--no-tokens", "demo.lune"]);
        assert_eq!(
            parse_args(&args),
            Ok(Command::Run {
                source_path: "demo.lune".to_string(),
                print_tokens: false
            })
        );
    }

    #[test]
    fn rejects_unknown_option() {
        let args = str_args(&["--wat"]);
        assert_eq!(parse_args(&args), Err("error: unknown option"));
    }

    #[test]
    fn reports_read_errors() {
        let args = str_args(&["missing.lune"]);
        let result = run_cli(&args, |_| Err(io::Error::from(io::ErrorKind::NotFound)));

        assert_eq!(result.exit_code, 1);
        assert!(result.stdout.is_empty());
        assert!(result.stderr.starts_with("error: "));
    }

    #[test]
    fn runs_pipeline_and_can_hide_tokens() {
        let args = str_args(&["--no-tokens", "demo.lune"]);
        let result = run_cli(&args, |_| Ok("1".to_string()));

        assert_eq!(result.exit_code, 0);
        assert!(!result.stdout.contains("\t\"1\"\t"));
        assert!(result.stdout.contains("vm stack: [Number(1.0)]"));
        assert!(result.stderr.is_empty());
    }
}
