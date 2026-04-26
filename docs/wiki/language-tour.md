# Language Tour

This page documents what the current implementation recognizes and how it behaves.

## Important model note

Lune currently has **token-level projection**, not a full expression/statement parser. That means many language-looking forms are lexed, but only certain token kinds become runtime values.

## Supported lexical forms

### Literals

```lune
123
45.67
"text"
true
false
null
```

### Identifiers

```lune
name
_user_1
```

Identifiers are collected into AST as `Identifier(String)` and currently reported as unresolved by the resolver.

### Keywords recognized by lexer

```text
fn if else while const return true false null
```

### Operators and punctuation recognized by lexer

```text
( ) { } , . : + - * / % = := => == != < <= > >=
```

### Comments

Line comments are supported:

```lune
42 // comment
"x"
```

## What becomes runtime values

The bytecode compiler emits constants for:

- numbers
- strings
- booleans
- null

Identifiers are ignored during bytecode generation.

### Example: literal stack program

```lune
1
"hello"
false
null
```

Runtime stack result:

```text
[Number(1.0), String("hello"), Bool(false), Null]
```

### Example: mixed literals + identifiers

```lune
1
foo
2
bar
```

Behavior:

- resolver warnings for `foo`, `bar`
- VM stack still contains literal values: `1`, `2`

## Diagnostics behavior

### Lexer diagnostics

- unexpected `!` (unless part of `!=`)
- unexpected character (e.g. `@`)
- unterminated string

### Parser diagnostics

- invalid number literal conversion for `TokenType::Number` values that fail `f64` parsing

### VM diagnostics

- invalid constant index (should not happen in normal compiler output, but VM handles it defensively)

## Practical usage patterns

### Pattern 1: data-only scripts (works well now)

```lune
100
"region-us"
true
```

Use this to exercise end-to-end tokenization → compilation → VM stack evaluation.

### Pattern 2: syntax prototyping

```lune
fn add(a, b) { return a + b }
if x >= 10 { y := 1 }
```

Useful for lexer validation and token shape, even though these forms are not yet semantically executed.

## Non-goals (current stage)

The following are not implemented as executable language semantics yet:

- function declarations/calls
- variable binding and mutation semantics
- control-flow execution (`if`, `while`)
- operator expression evaluation
