# Getting Started with Lune

## 1) Build

From the repository root:

```bash
cmake -S . -B build -DCMAKE_BUILD_TYPE=Release
cmake --build build -j
```

This builds:

- `build/lune` (CLI)
- `build/lune_tests` (test binary, when tests are enabled)

## 2) Write your first program

Create `hello.lune`:

```lune
fn main() {
    x := 40
    x = x + 2
    return x
}
```

## 3) Run with JIT

```bash
./build/lune hello.lune --jit
```

`--jit` is optional because JIT execution is the default mode when `--aot` is not selected.

Expected output:

```text
42
```

## 4) Emit an object file (AOT)

```bash
./build/lune hello.lune --aot hello.o
```

Expected output:

```text
Wrote object file: hello.o
```

## 5) Run tests

```bash
ctest --test-dir build --output-on-failure
```

or directly:

```bash
./build/lune_tests
```
