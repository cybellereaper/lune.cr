const std = @import("std");
const Lexer = @import("lexer.zig").Lexer;
const Parser = @import("parser.zig").Parser;
const Interpreter = @import("interpreter.zig").Interpreter;
const pretty = @import("pretty_printer.zig");

fn runMain(source: []const u8) !f64 {
    var arena = std.heap.ArenaAllocator.init(std.testing.allocator);
    defer arena.deinit();
    const allocator = arena.allocator();

    var lexer = Lexer.init(allocator, source);
    const tokens = try lexer.tokenize();

    var parser = Parser.init(allocator, tokens);
    const program = try parser.parseProgram();

    var interpreter = Interpreter.init(allocator);
    defer interpreter.deinit();

    return interpreter.runMain(program);
}

test "main returns computed value" {
    const value = try runMain(
        \\fn main() {
        \\  x := 40
        \\  x = x + 2
        \\  return x
        \\}
    );
    try std.testing.expectEqual(@as(f64, 42), value);
}

test "while loop sums range" {
    const value = try runMain(
        \\fn main() {
        \\  i := 1
        \\  acc := 0
        \\  while i <= 5 {
        \\    acc = acc + i
        \\    i = i + 1
        \\  }
        \\  return acc
        \\}
    );
    try std.testing.expectEqual(@as(f64, 15), value);
}

test "if else chooses fallback branch" {
    const value = try runMain(
        \\fn main() {
        \\  x := 2
        \\  if x > 5 {
        \\    return 100
        \\  } else {
        \\    return 200
        \\  }
        \\}
    );
    try std.testing.expectEqual(@as(f64, 200), value);
}

test "function call and modulo work" {
    const value = try runMain(
        \\fn reduce(x, y) {
        \\  return (x % y) + 1
        \\}
        \\
        \\fn main() {
        \\  return reduce(20, 6)
        \\}
    );
    try std.testing.expectEqual(@as(f64, 3), value);
}

test "function calls parse expression arguments" {
    const value = try runMain(
        \\fn add(a, b) {
        \\  return a + b
        \\}
        \\
        \\fn main() {
        \\  return add(1 + 2, add(3, 4))
        \\}
    );
    try std.testing.expectEqual(@as(f64, 10), value);
}

test "parser rejects malformed function" {
    var arena = std.heap.ArenaAllocator.init(std.testing.allocator);
    defer arena.deinit();
    const allocator = arena.allocator();

    var lexer = Lexer.init(allocator, "fn main( { return 1 }");
    const tokens = try lexer.tokenize();
    var parser = Parser.init(allocator, tokens);

    try std.testing.expectError(error.ExpectedIdentifier, parser.parseProgram());
}

test "pretty printer emits readable source" {
    var arena = std.heap.ArenaAllocator.init(std.testing.allocator);
    defer arena.deinit();
    const allocator = arena.allocator();

    const source =
        \\const seed = 1
        \\fn main() {
        \\  x := seed + 1
        \\  return x
        \\}
    ;

    var lexer = Lexer.init(allocator, source);
    const tokens = try lexer.tokenize();
    var parser = Parser.init(allocator, tokens);
    const program = try parser.parseProgram();
    const rendered = try pretty.render(allocator, program);

    try std.testing.expect(std.mem.indexOf(u8, rendered, "fn main") != null);
    try std.testing.expect(std.mem.indexOf(u8, rendered, "return x") != null);
}
