use std::env;
use std::fs;

use lune::cli::{run_cli, CliFileSystem};

struct StdFileSystem;

impl CliFileSystem for StdFileSystem {
    fn read_to_string(&self, path: &str) -> std::io::Result<String> {
        fs::read_to_string(path)
    }

    fn write_string(&self, path: &str, content: &str) -> std::io::Result<()> {
        fs::write(path, content)
    }
}

fn main() {
    std::process::exit(run(env::args().skip(1).collect()));
}

fn run(argv: Vec<String>) -> i32 {
    let result = run_cli(&argv, &StdFileSystem);
    print!("{}", result.stdout);
    eprint!("{}", result.stderr);
    result.exit_code
}
