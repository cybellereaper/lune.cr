# Tooling and Execution

## CLI usage

```text
lune <file.lune>
```

Behavior:

- The CLI lexes the provided file.
- Tokens are written to stdout.
- Diagnostics are written to stderr and exit with status code `1`.

## Typical workflows

### Quick local run

```bash
crystal run src/lune.cr -- program.lune
```

### Build optimized binary

```bash
crystal build src/lune.cr --release -o bin/lune
```

## Diagnostics

- Lexer diagnostics include a message and line/column location.
- The CLI returns exit code `1` when diagnostics are present.
