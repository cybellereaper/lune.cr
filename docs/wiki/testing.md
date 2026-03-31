# Testing Guide

## What is covered

The Zig test suite currently validates lexer behavior:

- Keyword and identifier tokenization
- Numeric and operator tokenization
- Leading trivia capture (whitespace/comments)
- Diagnostic reporting for lexical errors

## Run tests

```bash
zig build test
```

or directly:

```bash
zig test src/lexer.zig
```

## Add new tests

1. Add a new `test "..." { ... }` block in `src/lexer.zig`.
2. Keep tests deterministic and independent.
3. Run `zig build test`.
