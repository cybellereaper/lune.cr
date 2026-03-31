# Tooling and Execution

## CLI usage

```text
lune <file.lune>
```

Behavior:

- The CLI lexes the provided file.
- Tokens are written to stdout.
- Diagnostics are written to stderr and exit with an error status.

## Typical workflows

### Quick local run

```bash
zig build run -- program.lune
```

### Build optimized binary

```bash
zig build -Doptimize=ReleaseFast
```

## Diagnostics

- Lexer diagnostics include a message and line/column location.
- `LexerDiagnosticsReported` is returned when diagnostics are present.
