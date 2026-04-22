# Testing Guide

## What is covered

The Crystal spec suite currently validates lexer behavior:

- Keyword and identifier tokenization
- Numeric and operator tokenization
- Leading trivia capture (whitespace/comments)
- Diagnostic reporting for lexical errors

## Run tests

```bash
crystal spec
```

## Add new tests

1. Add a new `it "..." do ... end` block in `spec/lexer_spec.cr`.
2. Keep specs deterministic and independent.
3. Run `crystal spec`.
