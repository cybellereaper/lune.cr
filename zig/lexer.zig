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

pub const DiagnosticKind = enum {
    unexpected_token_bang,
    unexpected_character,
    unterminated_string,

    pub fn message(kind: DiagnosticKind) []const u8 {
        return switch (kind) {
            .unexpected_token_bang => "Unexpected token !",
            .unexpected_character => "Unexpected character in input",
            .unterminated_string => "Unterminated string",
        };
    }
};

pub const Diagnostic = struct {
    kind: DiagnosticKind,
    line: usize,
    column: usize,

    pub fn message(self: Diagnostic) []const u8 {
        return self.kind.message();
    }
};

const Scanner = struct {
    source: []const u8,
    offset: usize = 0,
    line: usize = 1,
    column: usize = 1,

    fn isAtEnd(self: *const Scanner) bool {
        return self.offset >= self.source.len;
    }

    fn peek(self: *const Scanner) ?u8 {
        if (self.isAtEnd()) return null;
        return self.source[self.offset];
    }

    fn peekNext(self: *const Scanner) ?u8 {
        const index = self.offset + 1;
        if (index >= self.source.len) return null;
        return self.source[index];
    }

    fn advance(self: *Scanner) ?u8 {
        if (self.isAtEnd()) return null;
        const char = self.source[self.offset];
        self.offset += 1;
        if (char == '\n') {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        return char;
    }

    fn match(self: *Scanner, expected: u8) bool {
        if (self.peek() != expected) return false;
        _ = self.advance();
        return true;
    }
};

const TokenStart = struct {
    offset: usize,
    line: usize,
    column: usize,
};

pub const Lexer = struct {
    allocator: std.mem.Allocator,
    scanner: Scanner,

    pub const Result = struct {
        tokens: std.ArrayList(Token),
        diagnostics: std.ArrayList(Diagnostic),

        pub fn deinit(self: *Result, allocator: std.mem.Allocator) void {
            self.tokens.deinit(allocator);
            self.diagnostics.deinit(allocator);
        }
    };

    pub fn init(allocator: std.mem.Allocator, source: []const u8) Lexer {
        return .{ .allocator = allocator, .scanner = .{ .source = source } };
    }

    pub fn tokenize(self: *Lexer) !Result {
        var result = Result{ .tokens = .empty, .diagnostics = .empty };
        errdefer result.deinit(self.allocator);

        while (!self.scanner.isAtEnd()) {
            const trivia_start = self.scanner.offset;
            self.skipTrivia();
            const trivia = self.scanner.source[trivia_start..self.scanner.offset];

            if (self.scanner.isAtEnd()) break;

            const start = TokenStart{
                .offset = self.scanner.offset,
                .line = self.scanner.line,
                .column = self.scanner.column,
            };
            const current = self.scanner.advance().?;
            try self.scanToken(&result, start, trivia, current);
        }

        try result.tokens.append(self.allocator, .{
            .token_type = .end,
            .lexeme = "",
            .leading_trivia = "",
            .line = self.scanner.line,
            .column = self.scanner.column,
        });
        return result;
    }

    fn scanToken(self: *Lexer, result: *Result, start: TokenStart, trivia: []const u8, current: u8) !void {
        if (singleCharacterToken(current)) |token_type| return self.appendToken(result, token_type, trivia, start, self.scanner.offset);

        switch (current) {
            ':' => {
                if (self.scanner.match('=')) return self.appendToken(result, .short_decl, trivia, start, self.scanner.offset);
                return self.appendToken(result, .colon, trivia, start, self.scanner.offset);
            },
            '=' => {
                if (self.scanner.match('=')) return self.appendToken(result, .eq, trivia, start, self.scanner.offset);
                if (self.scanner.match('>')) return self.appendToken(result, .arrow, trivia, start, self.scanner.offset);
                return self.appendToken(result, .assign, trivia, start, self.scanner.offset);
            },
            '!' => {
                if (self.scanner.match('=')) return self.appendToken(result, .ne, trivia, start, self.scanner.offset);
                return self.appendDiagnostic(result, .unexpected_token_bang, start);
            },
            '<' => {
                if (self.scanner.match('=')) return self.appendToken(result, .le, trivia, start, self.scanner.offset);
                return self.appendToken(result, .lt, trivia, start, self.scanner.offset);
            },
            '>' => {
                if (self.scanner.match('=')) return self.appendToken(result, .ge, trivia, start, self.scanner.offset);
                return self.appendToken(result, .gt, trivia, start, self.scanner.offset);
            },
            '"' => return self.scanString(result, start, trivia),
            else => {},
        }

        if (std.ascii.isDigit(current)) return self.scanNumber(result, start, trivia);
        if (isIdentifierStart(current)) return self.scanIdentifier(result, start, trivia);
        return self.appendDiagnostic(result, .unexpected_character, start);
    }

    fn scanIdentifier(self: *Lexer, result: *Result, start: TokenStart, trivia: []const u8) !void {
        while (self.scanner.peek()) |next| {
            if (!isIdentifierPart(next)) break;
            _ = self.scanner.advance();
        }

        const lexeme = self.scanner.source[start.offset..self.scanner.offset];
        try result.tokens.append(self.allocator, .{
            .token_type = keywordType(lexeme),
            .lexeme = lexeme,
            .leading_trivia = trivia,
            .line = start.line,
            .column = start.column,
        });
    }

    fn scanNumber(self: *Lexer, result: *Result, start: TokenStart, trivia: []const u8) !void {
        while (self.scanner.peek()) |next| {
            if (!std.ascii.isDigit(next)) break;
            _ = self.scanner.advance();
        }

        if (self.scanner.peek() == '.') {
            if (self.scanner.peekNext()) |fraction_start| {
                if (std.ascii.isDigit(fraction_start)) {
                    _ = self.scanner.advance();
                    while (self.scanner.peek()) |digit| {
                        if (!std.ascii.isDigit(digit)) break;
                        _ = self.scanner.advance();
                    }
                }
            }
        }

        return self.appendToken(result, .number, trivia, start, self.scanner.offset);
    }

    fn scanString(self: *Lexer, result: *Result, start: TokenStart, trivia: []const u8) !void {
        const string_content_start = self.scanner.offset;

        while (self.scanner.peek()) |next| {
            if (next == '"') break;
            _ = self.scanner.advance();
        }

        if (self.scanner.isAtEnd()) {
            try result.tokens.append(self.allocator, .{
                .token_type = .string,
                .lexeme = self.scanner.source[string_content_start..self.scanner.offset],
                .leading_trivia = trivia,
                .line = start.line,
                .column = start.column,
            });
            return self.appendDiagnostic(result, .unterminated_string, start);
        }

        const string_content_end = self.scanner.offset;
        _ = self.scanner.advance();

        try result.tokens.append(self.allocator, .{
            .token_type = .string,
            .lexeme = self.scanner.source[string_content_start..string_content_end],
            .leading_trivia = trivia,
            .line = start.line,
            .column = start.column,
        });
    }

    fn skipTrivia(self: *Lexer) void {
        while (!self.scanner.isAtEnd()) {
            const current = self.scanner.peek().?;
            if (std.ascii.isWhitespace(current)) {
                _ = self.scanner.advance();
                continue;
            }
            if (current == '/' and self.scanner.peekNext() == '/') {
                _ = self.scanner.advance();
                _ = self.scanner.advance();
                while (self.scanner.peek()) |next| {
                    if (next == '\n') break;
                    _ = self.scanner.advance();
                }
                continue;
            }
            return;
        }
    }

    fn appendToken(self: *Lexer, result: *Result, token_type: TokenType, trivia: []const u8, start: TokenStart, end_offset: usize) !void {
        try result.tokens.append(self.allocator, .{
            .token_type = token_type,
            .lexeme = self.scanner.source[start.offset..end_offset],
            .leading_trivia = trivia,
            .line = start.line,
            .column = start.column,
        });
    }

    fn appendDiagnostic(self: *Lexer, result: *Result, kind: DiagnosticKind, start: TokenStart) !void {
        try result.diagnostics.append(self.allocator, .{ .kind = kind, .line = start.line, .column = start.column });
    }
};

fn singleCharacterToken(char: u8) ?TokenType {
    return switch (char) {
        '(' => .l_paren,
        ')' => .r_paren,
        '{' => .l_brace,
        '}' => .r_brace,
        ',' => .comma,
        '.' => .dot,
        '+' => .plus,
        '-' => .minus,
        '*' => .star,
        '/' => .slash,
        '%' => .percent,
        else => null,
    };
}

fn isIdentifierStart(char: u8) bool {
    return std.ascii.isAlphabetic(char) or char == '_';
}

fn isIdentifierPart(char: u8) bool {
    return isIdentifierStart(char) or std.ascii.isDigit(char);
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

test "tokenizes keywords operators numbers and trivia" {
    const allocator = std.testing.allocator;
    var lexer = Lexer.init(allocator, "  const value := 42.5 // note\nreturn value");
    var result = try lexer.tokenize();
    defer result.deinit(allocator);

    try std.testing.expectEqual(@as(usize, 7), result.tokens.items.len);
    try std.testing.expectEqual(TokenType.kw_const, result.tokens.items[0].token_type);
    try std.testing.expectEqualStrings("  ", result.tokens.items[0].leading_trivia);
    try std.testing.expectEqualStrings("const", result.tokens.items[0].lexeme);

    try std.testing.expectEqual(TokenType.identifier, result.tokens.items[1].token_type);
    try std.testing.expectEqualStrings("value", result.tokens.items[1].lexeme);

    try std.testing.expectEqual(TokenType.short_decl, result.tokens.items[2].token_type);
    try std.testing.expectEqualStrings(":=", result.tokens.items[2].lexeme);

    try std.testing.expectEqual(TokenType.number, result.tokens.items[3].token_type);
    try std.testing.expectEqualStrings("42.5", result.tokens.items[3].lexeme);

    try std.testing.expectEqual(TokenType.kw_return, result.tokens.items[4].token_type);
    try std.testing.expectEqualStrings(" // note\n", result.tokens.items[4].leading_trivia);

    try std.testing.expectEqual(TokenType.identifier, result.tokens.items[5].token_type);
    try std.testing.expectEqualStrings("value", result.tokens.items[5].lexeme);

    try std.testing.expectEqual(TokenType.end, result.tokens.items[6].token_type);
    try std.testing.expectEqual(@as(usize, 0), result.diagnostics.items.len);
}

test "reports unexpected bang token" {
    const allocator = std.testing.allocator;
    var lexer = Lexer.init(allocator, "!");
    var result = try lexer.tokenize();
    defer result.deinit(allocator);

    try std.testing.expectEqual(@as(usize, 1), result.diagnostics.items.len);
    try std.testing.expectEqual(DiagnosticKind.unexpected_token_bang, result.diagnostics.items[0].kind);
    try std.testing.expectEqualStrings("Unexpected token !", result.diagnostics.items[0].message());
    try std.testing.expectEqual(@as(usize, 1), result.tokens.items.len);
    try std.testing.expectEqual(TokenType.end, result.tokens.items[0].token_type);
}

test "reports unterminated string and keeps parsed text" {
    const allocator = std.testing.allocator;
    var lexer = Lexer.init(allocator, "\"hello");
    var result = try lexer.tokenize();
    defer result.deinit(allocator);

    try std.testing.expectEqual(@as(usize, 2), result.tokens.items.len);
    try std.testing.expectEqual(TokenType.string, result.tokens.items[0].token_type);
    try std.testing.expectEqualStrings("hello", result.tokens.items[0].lexeme);

    try std.testing.expectEqual(@as(usize, 1), result.diagnostics.items.len);
    try std.testing.expectEqual(DiagnosticKind.unterminated_string, result.diagnostics.items[0].kind);
    try std.testing.expectEqualStrings("Unterminated string", result.diagnostics.items[0].message());
}

test "parses comparison and arrow operators" {
    const allocator = std.testing.allocator;
    var lexer = Lexer.init(allocator, "a==b != c <= d >= e => f");
    var result = try lexer.tokenize();
    defer result.deinit(allocator);

    const expected = [_]TokenType{ .identifier, .eq, .identifier, .ne, .identifier, .le, .identifier, .ge, .identifier, .arrow, .identifier, .end };
    try std.testing.expectEqual(expected.len, result.tokens.items.len);
    for (expected, 0..) |token_type, index| {
        try std.testing.expectEqual(token_type, result.tokens.items[index].token_type);
    }
    try std.testing.expectEqual(@as(usize, 0), result.diagnostics.items.len);
}
