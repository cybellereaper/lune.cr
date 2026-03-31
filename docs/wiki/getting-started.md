# Getting Started with Lune

## 1) Build

From the repository root:

```bash
zig build
```

This builds and installs:

- `zig-out/bin/lune` (CLI)

## 2) Write your first program

Create `hello.lune`:

```lune
fn main() {
    value := 40
    value = value + 2
    return value
}
```

## 3) Tokenize source

```bash
zig build run -- hello.lune
```

Expected output is a token stream with line/column positions.

## 4) Run tests

```bash
zig build test
```
