use std::env;
use std::fs;

use lune::pipeline::run_pipeline;
use lune::{Token, TokenType, TOKEN_TYPE_NAMES};

fn main() {
    std::process::exit(run(env::args().skip(1).collect()));
}

fn run(argv: Vec<String>) -> i32 {
    if argv.len() != 1 {
        eprintln!("Usage: lune <file.lune>");
        return 1;
    }

    let source_path = &argv[0];
    let source = match fs::read_to_string(source_path) {
        Ok(content) => content,
        Err(error) => {
            eprintln!("error: {error}");
            return 1;
        }
    };

    let output = run_pipeline(&source);
    print_tokens(&output.lexed.tokens);

    for diagnostic in &output.lexed.diagnostics {
        eprintln!(
            "lexer error: {} at {}:{}",
            diagnostic.message(),
            diagnostic.line,
            diagnostic.column
        );
    }

    for diagnostic in &output.parsed.diagnostics {
        eprintln!(
            "parser error: {} at {}:{}",
            diagnostic.kind.message(),
            diagnostic.line,
            diagnostic.column
        );
    }

    for diagnostic in &output.resolved.diagnostics {
        eprintln!(
            "resolver warning: {} ({})",
            diagnostic.kind.message(),
            diagnostic.name
        );
    }

    for diagnostic in &output.vm_result.diagnostics {
        eprintln!(
            "vm error: {} at instruction {} (constant index {})",
            diagnostic.kind.message(),
            diagnostic.instruction_offset,
            diagnostic.constant_index
        );
    }

    if output.lexed.diagnostics.is_empty() {
        println!("vm stack: {:?}", output.vm_result.stack);
    }

    if output.lexed.diagnostics.is_empty()
        && output.parsed.diagnostics.is_empty()
        && output.vm_result.diagnostics.is_empty()
    {
        0
    } else {
        1
    }
}

fn print_tokens(tokens: &[Token]) {
    for token in tokens {
        let token_name = token_name(token.token_type);
        println!(
            "{}\t\"{}\"\t({}:{})",
            token_name, token.lexeme, token.line, token.column
        );
    }
}

fn token_name(token_type: TokenType) -> &'static str {
    TOKEN_TYPE_NAMES
        .iter()
        .find_map(|(kind, name)| (*kind == token_type).then_some(*name))
        .unwrap_or("unknown")
}
