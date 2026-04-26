use std::env;
use std::fs;

use lune::cli::run_cli;

fn main() {
    std::process::exit(run(env::args().skip(1).collect()));
}

fn run(argv: Vec<String>) -> i32 {
    let result = run_cli(&argv, |path| fs::read_to_string(path));
    print!("{}", result.stdout);
    eprint!("{}", result.stderr);
    result.exit_code
}
