const std = @import("std");
const lexer_module = @import("lexer.zig");

pub fn main() !void {
    var general_purpose_allocator = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = general_purpose_allocator.deinit();

    const allocator = general_purpose_allocator.allocator();
    const args = try std.process.argsAlloc(allocator);
    defer std.process.argsFree(allocator, args);

    if (args.len != 2) {
        std.debug.print("Usage: lune <file.lune>\n", .{});
        return;
    }

    const source = try std.fs.cwd().readFileAlloc(allocator, args[1], 1024 * 1024 * 4);
    defer allocator.free(source);

    var lexer = lexer_module.Lexer.init(allocator, source);
    var result = try lexer.tokenize();
    defer result.deinit(allocator);

    const stdout = std.fs.File.stdout().deprecatedWriter();
    for (result.tokens.items) |token| {
        try stdout.print("{s}\t\"{s}\"\t({d}:{d})\n", .{
            lexer_module.tokenTypeName(token.token_type),
            token.lexeme,
            token.line,
            token.column,
        });
    }

    if (result.diagnostics.items.len == 0) return;

    const stderr = std.fs.File.stderr().deprecatedWriter();
    for (result.diagnostics.items) |diagnostic| {
        try stderr.print("error: {s} at {d}:{d}\n", .{ diagnostic.message(), diagnostic.line, diagnostic.column });
    }
    return error.LexerDiagnosticsReported;
}
