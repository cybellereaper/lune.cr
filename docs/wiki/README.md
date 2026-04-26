# Lune Documentation

Welcome to the docs for the Rust implementation of **Lune** in this repository.

> Status: this is an early-stage language/runtime prototype. The current pipeline is:
> **Lexer → Parser (literal/identifier projection) → Resolver (identifier warnings) → Bytecode → VM**.

## Documentation map

- [Getting Started](./getting-started.md) — install, run, and read CLI output.
- [Language Tour](./language-tour.md) — syntax currently recognized by the lexer and projected into runtime values.
- [Tooling and Execution](./tooling-and-execution.md) — CLI behavior, exit codes, and practical workflows.
- [Testing Guide](./testing.md) — test commands, scope, and how to add coverage.

## Quick example

Create `sample.lune`:

```lune
42 "hello" true null user
```

Run:

```bash
cargo run -- sample.lune
```

You will see:

1. A token stream for all lexed tokens.
2. Resolver warnings for identifiers (for example `user`).
3. A final VM stack with literal values (number/string/bool/null).

## What works today

- ASCII lexer with keyword/operator/token support.
- Line/column diagnostics from lexer and parser.
- Parser projection of `number`, `string`, `true`, `false`, `null`, and identifiers.
- Bytecode compiler that emits `PushConst` for literal nodes.
- VM execution of `PushConst` and `Halt`.

## What is intentionally incomplete

- No expression grammar or statement semantics in parser yet (tokens are projected node-by-node).
- No symbol table/declaration resolution yet (identifiers are reported as unresolved).
- No control-flow/runtime instruction set beyond constant pushes.

If you are new to the repo, start with [Getting Started](./getting-started.md).
