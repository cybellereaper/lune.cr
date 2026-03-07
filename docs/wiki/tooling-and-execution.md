# Tooling and Execution

## CLI usage

```text
lune <file.lune> [--jit|--aot <out.o>]
```

Behavior:

- If `--aot` is provided, Lune emits an object file and exits.
- Otherwise Lune JIT-compiles and runs `main()`, then prints the return value.

## Typical workflows

### Quick JIT loop

```bash
./build/lune program.lune
```

or explicitly:

```bash
./build/lune program.lune --jit
```

### Generate object code

```bash
./build/lune program.lune --aot program.o
```

## Build options

`CMakeLists.txt` exposes:

- `LUNE_BUILD_TESTS=ON|OFF` (default `ON`)

Example disabling tests:

```bash
cmake -S . -B build -DLUNE_BUILD_TESTS=OFF
cmake --build build -j
```

## Diagnostics

- Lexing and parsing collect diagnostics (line/column).
- The CLI catches exceptions and prints `error: <message>`.

## Pretty printing

The codebase includes a pretty-printer utility (`pretty_print`) that renders AST back to source-like text. This is used in tests and is useful when inspecting parse output in development.
