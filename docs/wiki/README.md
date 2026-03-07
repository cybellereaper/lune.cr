# Lune Wiki

This wiki is a practical guide to the current Lune language implementation in this repository.

## Pages

- [Getting Started](./getting-started.md)
- [Language Tour](./language-tour.md)
- [Tooling and Execution](./tooling-and-execution.md)
- [Testing Guide](./testing.md)

## What Lune currently supports

Lune is a small expression-oriented language with:

- Numeric values (`double` at runtime)
- `true`/`false`/`null`
- Variables with `const name = expr` and `name := expr`
- Re-assignment via `name = expr`
- `if`/`else`, `while`, `return`
- Function declarations and function calls
- JIT execution of `main()` and AOT object file emission

For exact syntax and examples, continue with [Language Tour](./language-tour.md).
