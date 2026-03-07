# Lune Language Tour

This page documents the currently implemented syntax and behavior.

## Program shape

A file is parsed as a sequence of declarations/statements. In practice, provide a `main` function for execution:

```lune
fn main() {
    return 0
}
```

## Values and literals

### Numbers

Numbers are floating-point (`double`) internally:

```lune
x := 1
y := 2.5
z := x + y
```

### Booleans

```lune
ok := true
fail := false
```

Booleans are represented as numeric `1.0` (`true`) and `0.0` (`false`) during code generation.

### Null

```lune
n := null
```

`null` currently lowers to `0.0` at runtime.

### Strings

```lune
msg := "hello"
```

Strings are tokenized and parsed, but runtime string semantics are not implemented yet (currently codegen lowers string expressions to `0.0`).

## Variables and assignment

### Constant declaration

```lune
const seed = 123
```

### Short declaration

```lune
x := 10
```

### Assignment

```lune
a := 1
a = a + 5
```

## Arithmetic and comparison operators

Supported binary operators:

- Arithmetic: `+`, `-`, `*`, `/`, `%`
- Comparison: `==`, `!=`, `<`, `<=`, `>`, `>=`

Example:

```lune
fn main() {
    a := 20
    b := 6
    mod := a % b
    ok := mod == 2
    if ok {
        return 1
    } else {
        return 0
    }
}
```

## Control flow

### if / else

```lune
fn main() {
    x := 3
    if x > 2 {
        return 10
    } else {
        return 20
    }
}
```

### while

```lune
fn main() {
    i := 1
    sum := 0
    while i <= 5 {
        sum = sum + i
        i = i + 1
    }
    return sum
}
```

## Functions and calls

Define and call functions with positional arguments:

```lune
fn add(a, b) {
    return a + b
}

fn main() {
    return add(10, 32)
}
```

## Comments

Line comments are supported using `//`:

```lune
fn main() {
    // this is ignored by the lexer
    return 42
}
```

## Notes and current limitations

- No logical operators like `&&`/`||`.
- No unary `!`.
- `if` exists as a statement; `IfExpr` is present in the AST but not lowered in codegen.
- Calls currently require a named callee (`identifier(args...)`).
- Strings and `null` are parsed but not fully typed/runtime-backed.
