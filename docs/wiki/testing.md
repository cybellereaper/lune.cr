# Testing Guide

## What is covered

The `lune_tests` binary currently validates:

- Lexer tokenization and diagnostics
- Parser shape and recovery behavior
- JIT execution (`main`, arithmetic, loops, function calls)
- AOT object-file emission
- Pretty-printer output
- GC mark/sweep behavior
- Basic performance sanity checks

## Run tests

After building:

```bash
./build/lune_tests
```

or via CTest:

```bash
ctest --test-dir build --output-on-failure
```

## Add new tests

1. Add a `void test_*()` function in `tests/lune_tests.cpp`.
2. Keep tests deterministic and avoid external dependencies.
3. Register the test call from `main()`.
4. Rebuild and run `./build/lune_tests`.

## Recommended areas for future tests

- More parser edge cases (nested blocks, invalid call syntax)
- AOT object validation by linking and executing a small program
- Runtime semantics once string/null typing is expanded
