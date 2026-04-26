# Tooling and Execution

## CLI contract

Command:

```text
lune <file.lune>
```

In this repository, the easiest invocation during development is:

```bash
cargo run -- <file.lune>
```

## What the CLI does

For a single input file, CLI runs:

1. Lexing
2. Parsing
3. Resolving
4. Bytecode compilation
5. VM execution

It prints token stream first, then diagnostics, then VM stack (when lexing succeeds).

## Local workflows

### Fast iteration

```bash
cargo run -- examples/sample.lune
```

### Build release binary

```bash
cargo build --release
```

Binary path:

```text
target/release/lune
```

Run binary:

```bash
./target/release/lune path/to/file.lune
```

### Validate code health

```bash
cargo test
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

## Output format reference

### Token line

```text
<token_name>    "<lexeme>"    (<line>:<column>)
```

### Diagnostic lines

```text
lexer error: <message> at <line>:<column>
parser error: <message> at <line>:<column>
resolver warning: <message> (<identifier>)
vm error: <message> at instruction <offset> (constant index <idx>)
```

### Stack line

```text
vm stack: [<Value>, ...]
```

## Exit behavior summary

The process returns exit code `1` if any of these occur:

- invalid CLI usage
- file read error
- any lexer diagnostic
- any parser diagnostic
- any VM diagnostic

Otherwise returns `0`.

## Example session

Input (`demo.lune`):

```lune
7
"ok"
true
who
```

Typical result:

- tokens for all lexemes
- resolver warning for `who`
- stack containing `Number(7.0)`, `String("ok")`, `Bool(true)`
