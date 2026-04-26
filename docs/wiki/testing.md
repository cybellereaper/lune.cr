# Testing Guide

## Test scope in this repository

The test suite currently covers each pipeline stage with unit tests colocated in source modules:

- `lexer.rs` — tokenization, trivia, diagnostics
- `cli.rs` — argument parsing, CLI output contract, read-error handling
- `parser.rs` — AST projection + parser diagnostics
- `resolver.rs` — unresolved identifier diagnostics
- `bytecode.rs` — literal-to-bytecode compilation
- `vm.rs` — instruction execution + VM diagnostics
- `pipeline.rs` — stage integration and short-circuit behavior

## Run tests

```bash
cargo test
```

## Run focused tests

```bash
cargo test lexer
cargo test parser
cargo test pipeline
```

## Recommended local quality checks

```bash
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

## How to add tests effectively

### 1) Unit tests near the code

Follow existing style:

- place tests under `#[cfg(test)] mod tests`
- keep test names behavior-focused
- assert both happy path and diagnostics

### 2) Cover edge cases first

Examples of valuable edge coverage:

- lexer: unterminated strings, unknown characters, tricky operator pairs (`=`, `==`, `=>`)
- parser: malformed numeric token payloads
- vm: invalid constant indexes and halt behavior

### 3) Keep assertions specific

Assert exact:

- token type sequences
- diagnostic kinds and positions
- stack values and instruction behavior

## Example minimal regression test checklist

When changing parser/bytecode/vm behavior, include tests for:

- valid literal handling
- invalid input diagnostics
- no panic/crash on malformed data
- preserved output contract for CLI-facing structures
