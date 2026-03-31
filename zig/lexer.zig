const std = @import("std");

pub const TokenType = enum {
    end,
    identifier,
    number,
    string,
    kw_fn,
    kw_if,
    kw_else,
    kw_while,
    kw_const,
    kw_return,
    kw_true,
    kw_false,
    kw_null,
    l_paren,
    r_paren,
    l_brace,
    r_brace,
    comma,
    dot,
    colon,
    plus,
    minus,
    star,
    slash,
    percent,
    assign,
    short_decl,
    arrow,
    eq,
    ne,
    lt,
    le,
    gt,
    ge,
};

pub const Token = struct {
    token_type: TokenType,
    lexeme: []const u8,
    leading_trivia: []const u8,
    line: usize,
    column: usize,
};

pub const Diagnostic = struct {
    message: []const u8,
    line: usize,
    column: usize,
};

const ScannerPosition = struct {
    offset: usize = 0,
    line: usize = 1,
    column: usize = 1,
};

pub const Lexer = struct {
    allocator: std.mem.Allocator,
    source: []const u8,
    position: ScannerPosition = .{},

    pub const Result = struct {
        tokens: std.ArrayList(Token),
        diagnostics: std.ArrayList(Diagnostic),

        pub fn deinit(self: *Result, allocator: std.mem.Allocator) void {
            for (self.tokens.items) |token| {
                allocator.free(token.lexeme);
                allocator.free(token.leading_trivia);
            }
            self.tokens.deinit(allocator);
            self.diagnostics.deinit(allocator);
        }
    };

    pub fn init(allocator: std.mem.Allocator, source: []const u8) Lexer {
        return .{ .allocator = allocator, .source = source };
    }

    pub fn tokenize(self: *Lexer) !Result {
        var result = Result{ .tokens = .empty, .diagnostics = .empty };
        errdefer result.deinit(self.allocator);

        while (!self.isAtEnd()) {
            const leading_trivia = try self.collectTrivia();
            if (self.isAtEnd()) {
                self.allocator.free(leading_trivia);
                break;
            }

            const token_start = self.position;
            const current = self.advance().?;
            try self.scanToken(current, token_start, leading_trivia, &result);
        }

        try result.tokens.append(self.allocator, .{
            .token_type = .end,
            .lexeme = try self.allocator.dupe(u8, ""),
            .leading_trivia = try self.allocator.dupe(u8, ""),
            .line = self.position.line,
            .column = self.position.column,
        });
        return result;
    }

    fn scanToken(self: *Lexer, current: u8, token_start: ScannerPosition, leading_trivia: []const u8, result: *Result) !void {
        const single_char = switch (current) {
            '(' => TokenType.l_paren,
            ')' => TokenType.r_paren,
            '{' => TokenType.l_brace,
            '}' => TokenType.r_brace,
            ',' => TokenType.comma,
            '.' => TokenType.dot,
            '+' => TokenType.plus,
            '-' => TokenType.minus,
            '*' => TokenType.star,
            '/' => TokenType.slash,
            '%' => TokenType.percent,
            else => null,
        };
        if (single_char) |token_type| return self.appendToken(result, token_type, &[_]u8{current}, leading_trivia, token_start);

        switch (current) {
            ':' => if (self.match('=')) {
                return self.appendToken(result, .short_decl, ":=", leading_trivia, token_start);
            } else {
                return self.appendToken(result, .colon, ":", leading_trivia, token_start);
            },
            '=' => {
                if (self.match('=')) return self.appendToken(result, .eq, "==", leading_trivia, token_start);
                if (self.match('>')) return self.appendToken(result, .arrow, "=>", leading_trivia, token_start);
                return self.appendToken(result, .assign, "=", leading_trivia, token_start);
            },
            '!' => {
                if (self.match('=')) return self.appendToken(result, .ne, "!=", leading_trivia, token_start);
                self.allocator.free(leading_trivia);
                try result.diagnostics.append(self.allocator, .{ .message = "Unexpected token !", .line = token_start.line, .column = token_start.column });
                return;
            },
            '<' => if (self.match('=')) {
                return self.appendToken(result, .le, "<=", leading_trivia, token_start);
            } else {
                return self.appendToken(result, .lt, "<", leading_trivia, token_start);
            },
            '>' => if (self.match('=')) {
                return self.appendToken(result, .ge, ">=", leading_trivia, token_start);
            } else {
                return self.appendToken(result, .gt, ">", leading_trivia, token_start);
            },
            '"' => return self.scanString(result, token_start, leading_trivia),
            else => {},
        }

        if (std.ascii.isDigit(current)) return self.scanNumber(result, token_start, leading_trivia, current);
        if (isIdentifierStart(current)) return self.scanIdentifier(result, token_start, leading_trivia, current);

        self.allocator.free(leading_trivia);
        try result.diagnostics.append(self.allocator, .{ .message = "Unexpected character in input", .line = token_start.line, .column = token_start.column });
    }

    fn scanIdentifier(self: *Lexer, result: *Result, token_start: ScannerPosition, leading_trivia: []const u8, first: u8) !void {
        var lexeme: std.ArrayList(u8) = .empty;
        defer lexeme.deinit(self.allocator);

        try lexeme.append(self.allocator, first);
        while (self.peek()) |next| {
            if (!isIdentifierPart(next)) break;
            _ = self.advance();
            try lexeme.append(self.allocator, next);
        }

        const lexeme_slice = try lexeme.toOwnedSlice(self.allocator);
        try result.tokens.append(self.allocator, .{
            .token_type = keywordType(lexeme_slice),
            .lexeme = lexeme_slice,
            .leading_trivia = leading_trivia,
            .line = token_start.line,
            .column = token_start.column,
        });
    }

    fn scanNumber(self: *Lexer, result: *Result, token_start: ScannerPosition, leading_trivia: []const u8, first: u8) !void {
        var lexeme: std.ArrayList(u8) = .empty;
        defer lexeme.deinit(self.allocator);

        try lexeme.append(self.allocator, first);
        while (self.peek()) |next| {
            if (!std.ascii.isDigit(next)) break;
            _ = self.advance();
            try lexeme.append(self.allocator, next);
        }

        if (self.peek() == '.') {
            if (self.peekNext()) |fraction_start| {
                if (std.ascii.isDigit(fraction_start)) {
                    _ = self.advance();
                    try lexeme.append(self.allocator, '.');
                    while (self.peek()) |digit| {
                        if (!std.ascii.isDigit(digit)) break;
                        _ = self.advance();
                        try lexeme.append(self.allocator, digit);
                    }
                }
            }
        }

        const number_lexeme = try lexeme.toOwnedSlice(self.allocator);
        try result.tokens.append(self.allocator, .{
            .token_type = .number,
            .lexeme = number_lexeme,
            .leading_trivia = leading_trivia,
            .line = token_start.line,
            .column = token_start.column,
        });
    }

    fn scanString(self: *Lexer, result: *Result, token_start: ScannerPosition, leading_trivia: []const u8) !void {
        const string_start = self.position.offset;
        while (self.peek()) |next| {
            if (next == '"') break;
            _ = self.advance();
        }

        if (self.isAtEnd()) {
            try result.tokens.append(self.allocator, .{
                .token_type = .string,
                .lexeme = try self.allocator.dupe(u8, self.source[string_start..]),
                .leading_trivia = leading_trivia,
                .line = token_start.line,
                .column = token_start.column,
            });
            try result.diagnostics.append(self.allocator, .{ .message = "Unterminated string", .line = token_start.line, .column = token_start.column });
            return;
        }

        const value = try self.allocator.dupe(u8, self.source[string_start..self.position.offset]);
        _ = self.advance();
        try result.tokens.append(self.allocator, .{
            .token_type = .string,
            .lexeme = value,
            .leading_trivia = leading_trivia,
            .line = token_start.line,
            .column = token_start.column,
        });
    }

    fn appendToken(self: *Lexer, result: *Result, token_type: TokenType, lexeme: []const u8, leading_trivia: []const u8, token_start: ScannerPosition) !void {
        try result.tokens.append(self.allocator, .{
            .token_type = token_type,
            .lexeme = try self.allocator.dupe(u8, lexeme),
            .leading_trivia = leading_trivia,
            .line = token_start.line,
            .column = token_start.column,
        });
    }

    fn collectTrivia(self: *Lexer) ![]const u8 {
        var trivia: std.ArrayList(u8) = .empty;
        defer trivia.deinit(self.allocator);

        while (!self.isAtEnd()) {
            const current = self.peek().?;
            if (std.ascii.isWhitespace(current)) {
                try trivia.append(self.allocator, self.advance().?);
                continue;
            }
            if (current == '/' and self.peekNext() == '/') {
                try trivia.append(self.allocator, self.advance().?);
                try trivia.append(self.allocator, self.advance().?);
                while (self.peek()) |next| {
                    if (next == '\n') break;
                    try trivia.append(self.allocator, self.advance().?);
                }
                continue;
            }
            break;
        }
        return trivia.toOwnedSlice(self.allocator);
    }

    fn isAtEnd(self: *const Lexer) bool {
        return self.position.offset >= self.source.len;
    }

    fn peek(self: *const Lexer) ?u8 {
        if (self.isAtEnd()) return null;
        return self.source[self.position.offset];
    }

    fn peekNext(self: *const Lexer) ?u8 {
        const next_index = self.position.offset + 1;
        if (next_index >= self.source.len) return null;
        return self.source[next_index];
    }

    fn match(self: *Lexer, expected: u8) bool {
        if (self.peek() != expected) return false;
        _ = self.advance();
        return true;
    }

    fn advance(self: *Lexer) ?u8 {
        if (self.isAtEnd()) return null;
        const value = self.source[self.position.offset];
        self.position.offset += 1;
        if (value == '\n') {
            self.position.line += 1;
            self.position.column = 1;
        } else {
            self.position.column += 1;
        }
        return value;
    }
};

fn isIdentifierStart(value: u8) bool {
    return std.ascii.isAlphabetic(value) or value == '_';
}

fn isIdentifierPart(value: u8) bool {
    return isIdentifierStart(value) or std.ascii.isDigit(value);
}

fn keywordType(lexeme: []const u8) TokenType {
    inline for ([_]struct { []const u8, TokenType }{
        .{ "fn", .kw_fn },
        .{ "if", .kw_if },
        .{ "else", .kw_else },
        .{ "while", .kw_while },
        .{ "const", .kw_const },
        .{ "return", .kw_return },
        .{ "true", .kw_true },
        .{ "false", .kw_false },
        .{ "null", .kw_null },
    }) |entry| {
        if (std.mem.eql(u8, lexeme, entry[0])) return entry[1];
    }
    return .identifier;
}

test "tokenizes keywords numbers and trivia" {
    const allocator = std.testing.allocator;
    var lexer = Lexer.init(allocator, "  const value := 42.5 // note\nreturn value");
    var result = try lexer.tokenize();
    defer result.deinit(allocator);

    try std.testing.expectEqual(@as(usize, 7), result.tokens.items.len);
    try std.testing.expectEqual(TokenType.kw_const, result.tokens.items[0].token_type);
    try std.testing.expectEqualStrings("  ", result.tokens.items[0].leading_trivia);
    try std.testing.expectEqual(TokenType.identifier, result.tokens.items[1].token_type);
    try std.testing.expectEqualStrings("value", result.tokens.items[1].lexeme);
    try std.testing.expectEqual(TokenType.short_decl, result.tokens.items[2].token_type);
    try std.testing.expectEqual(TokenType.number, result.tokens.items[3].token_type);
    try std.testing.expectEqualStrings("42.5", result.tokens.items[3].lexeme);
    try std.testing.expectEqual(TokenType.kw_return, result.tokens.items[4].token_type);
    try std.testing.expectEqualStrings(" // note\n", result.tokens.items[4].leading_trivia);
    try std.testing.expectEqual(TokenType.identifier, result.tokens.items[5].token_type);
    try std.testing.expectEqualStrings("value", result.tokens.items[5].lexeme);
    try std.testing.expectEqual(@as(usize, 0), result.diagnostics.items.len);
}

test "reports invalid exclamation token" {
    const allocator = std.testing.allocator;
    var lexer = Lexer.init(allocator, "!");
    var result = try lexer.tokenize();
    defer result.deinit(allocator);

    try std.testing.expectEqual(@as(usize, 1), result.diagnostics.items.len);
    try std.testing.expectEqualStrings("Unexpected token !", result.diagnostics.items[0].message);
    try std.testing.expectEqual(@as(usize, 1), result.tokens.items.len);
    try std.testing.expectEqual(TokenType.end, result.tokens.items[0].token_type);
}

test "keeps unterminated string and reports diagnostic" {
    const allocator = std.testing.allocator;
    var lexer = Lexer.init(allocator, "\"hello");
    var result = try lexer.tokenize();
    defer result.deinit(allocator);

    try std.testing.expectEqual(@as(usize, 2), result.tokens.items.len);
    try std.testing.expectEqual(TokenType.string, result.tokens.items[0].token_type);
    try std.testing.expectEqualStrings("hello", result.tokens.items[0].lexeme);
    try std.testing.expectEqual(@as(usize, 1), result.diagnostics.items.len);
    try std.testing.expectEqualStrings("Unterminated string", result.diagnostics.items[0].message);
}
