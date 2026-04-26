use std::env;
use std::fs;

use lune::{Lexer, Token, TokenType, TOKEN_TYPE_NAMES};

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

    let mut lexer = Lexer::new(&source);
    let result = lexer.tokenize();
    print_tokens(&result.tokens);

    if result.diagnostics.is_empty() {
        return 0;
    }

    for diagnostic in &result.diagnostics {
        eprintln!(
            "error: {} at {}:{}",
            diagnostic.message(),
            diagnostic.line,
            diagnostic.column
        );
    }

    1
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
