# Getting Started

This guide gets you from clone to running `.lune` files with the current Rust CLI.

## 1) Prerequisites

- Rust toolchain (stable)
- Cargo

Optional but useful:

- `rustfmt`
- `clippy`

## 2) Build and run tests

From repository root:

```bash
cargo test
```

## 3) Create a Lune file

Create `hello.lune`:

```lune
42
"hi"
true
null
name
```

### Why this example?

With the current implementation:

- literals (`42`, `"hi"`, `true`, `null`) become AST nodes, constants, then VM stack values.
- identifiers (`name`) are accepted lexically but reported by resolver as unresolved.

## 4) Run the CLI

```bash
cargo run -- hello.lune
```

## 5) Understand the output

You should expect output sections in this order:

1. **Tokens** (stdout), one per line with type, lexeme, and position.
2. **Diagnostics** (stderr), if present:
   - `lexer error: ...`
   - `parser error: ...`
   - `resolver warning: ...`
   - `vm error: ...`
3. **VM stack** (stdout) if lexing succeeded.

## 6) Exit codes

- `0`: no lexer/parser/vm errors.
- `1`: usage error, IO error, or any lexer/parser/vm diagnostics.

Resolver output is a warning stream and does not currently block successful execution by itself.

## Troubleshooting

### `Usage: lune <file.lune>`

You passed zero or multiple arguments. Provide exactly one source file.

### `error: ...` while reading file

Check path correctness and file permissions.

### You expected rich language execution

At this stage, Lune is intentionally minimal. See [Language Tour](./language-tour.md) for the exact current behavior.
