mod lexer;

use std::env;
use std::fs;

use lexer::Lexer;

fn main() {
    let mut args = env::args();
    let binary_name = args.next().unwrap_or_else(|| String::from("lune"));
    let Some(path) = args.next() else {
        eprintln!("Usage: {binary_name} <file.lune>");
        std::process::exit(1);
    };

    if args.next().is_some() {
        eprintln!("Usage: {binary_name} <file.lune>");
        std::process::exit(1);
    }

    let source = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) => {
            eprintln!("Failed to read {path}: {error}");
            std::process::exit(1);
        }
    };

    let mut lexer = Lexer::new(&source);
    let result = lexer.tokenize();

    for token in &result.tokens {
        println!(
            "{}\t\"{}\"\t({}:{})",
            token.token_type.display_name(),
            token.lexeme,
            token.line,
            token.column
        );
    }

    if result.diagnostics.is_empty() {
        return;
    }

    for diagnostic in &result.diagnostics {
        eprintln!(
            "error: {} at {}:{}",
            diagnostic.message(),
            diagnostic.line,
            diagnostic.column
        );
    }

    std::process::exit(1);
}
