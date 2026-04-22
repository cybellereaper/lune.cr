# Getting Started with Lune

## 1) Install dependencies

From the repository root:

```bash
shards install
```

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
crystal run src/lune.cr -- hello.lune
```

Expected output is a token stream with line/column positions.

## 4) Run tests

```bash
crystal spec
```
