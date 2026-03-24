const std = @import("std");
const ast = @import("ast.zig");
const token_mod = @import("token.zig");
const Token = token_mod.Token;
const TokenType = token_mod.TokenType;

const ParseError = error{
    ExpectedTopLevelItem,
    ExpectedIdentifier,
    ExpectedAssign,
    ExpectedLeftParen,
    ExpectedRightParen,
    ExpectedLeftBrace,
    ExpectedRightBrace,
    ExpectedExpression,
    InvalidBinaryOperator,
};

const ParseResult = ParseError || std.mem.Allocator.Error;

pub const Parser = struct {
    tokens: []const Token,
    index: usize = 0,
    allocator: std.mem.Allocator,

    pub fn init(allocator: std.mem.Allocator, tokens: []const Token) Parser {
        return .{ .allocator = allocator, .tokens = tokens };
    }

    pub fn parseProgram(self: *Parser) ParseResult!ast.Program {
        var items = std.ArrayList(ast.Item).init(self.allocator);
        while (!self.current().is(.eof)) {
            try items.append(try self.parseItem());
        }
        return .{ .items = try items.toOwnedSlice() };
    }

    fn parseItem(self: *Parser) ParseResult!ast.Item {
        if (self.match(.kw_const)) return .{ .const_decl = try self.parseConstDecl() };
        if (self.match(.kw_fn)) return .{ .function_decl = try self.parseFunction() };
        return error.ExpectedTopLevelItem;
    }

    fn parseConstDecl(self: *Parser) ParseResult!ast.ConstDecl {
        const name = try self.consume(.identifier, error.ExpectedIdentifier);
        _ = try self.consume(.assign, error.ExpectedAssign);
        const value = try self.parseExpression();
        return .{ .name = name.lexeme, .value = value };
    }

    fn parseFunction(self: *Parser) ParseResult!ast.FunctionDecl {
        const name = try self.consume(.identifier, error.ExpectedIdentifier);
        _ = try self.consume(.l_paren, error.ExpectedLeftParen);

        var params = std.ArrayList([]const u8).init(self.allocator);
        if (!self.current().is(.r_paren)) {
            while (true) {
                const param = try self.consume(.identifier, error.ExpectedIdentifier);
                try params.append(param.lexeme);
                if (!self.match(.comma)) break;
            }
        }

        _ = try self.consume(.r_paren, error.ExpectedRightParen);
        const body = try self.parseBlock();
        return .{ .name = name.lexeme, .params = try params.toOwnedSlice(), .body = body };
    }

    fn parseBlock(self: *Parser) ParseResult![]ast.Stmt {
        _ = try self.consume(.l_brace, error.ExpectedLeftBrace);
        var statements = std.ArrayList(ast.Stmt).init(self.allocator);
        while (!self.current().is(.r_brace) and !self.current().is(.eof)) {
            try statements.append(try self.parseStatement());
        }
        _ = try self.consume(.r_brace, error.ExpectedRightBrace);
        return try statements.toOwnedSlice();
    }

    fn parseStatement(self: *Parser) ParseResult!ast.Stmt {
        if (self.match(.kw_return)) {
            if (self.current().kind == .r_brace) return .{ .return_stmt = null };
            return .{ .return_stmt = try self.parseExpression() };
        }

        if (self.match(.kw_if)) return .{ .if_stmt = try self.parseIfStmt() };
        if (self.match(.kw_while)) return .{ .while_stmt = try self.parseWhileStmt() };

        if (self.current().kind == .identifier and self.peek().kind == .decl_assign) {
            const name = self.advance().lexeme;
            _ = self.advance();
            return .{ .var_decl = .{ .name = name, .value = try self.parseExpression() } };
        }

        if (self.current().kind == .identifier and self.peek().kind == .assign) {
            const name = self.advance().lexeme;
            _ = self.advance();
            return .{ .assign = .{ .name = name, .value = try self.parseExpression() } };
        }

        return .{ .expr_stmt = try self.parseExpression() };
    }

    fn parseIfStmt(self: *Parser) ParseResult!ast.Stmt.if_stmt {
        const condition = try self.parseExpression();
        const then_block = try self.parseBlock();
        var else_block: []ast.Stmt = &[_]ast.Stmt{};
        if (self.match(.kw_else)) else_block = try self.parseBlock();
        return .{ .condition = condition, .then_block = then_block, .else_block = else_block };
    }

    fn parseWhileStmt(self: *Parser) ParseResult!ast.Stmt.while_stmt {
        const condition = try self.parseExpression();
        const body = try self.parseBlock();
        return .{ .condition = condition, .body = body };
    }

    fn parseExpression(self: *Parser) ParseResult!*ast.Expr {
        return self.parseEquality();
    }

    fn parseEquality(self: *Parser) ParseResult!*ast.Expr {
        var expr = try self.parseComparison();
        while (self.match(.equal_equal) or self.match(.bang_equal)) {
            const op_token = self.previous();
            const right = try self.parseComparison();
            expr = try self.makeBinary(op_token.kind, expr, right);
        }
        return expr;
    }

    fn parseComparison(self: *Parser) ParseResult!*ast.Expr {
        var expr = try self.parseTerm();
        while (self.match(.less) or self.match(.less_equal) or self.match(.greater) or self.match(.greater_equal)) {
            const op_token = self.previous();
            const right = try self.parseTerm();
            expr = try self.makeBinary(op_token.kind, expr, right);
        }
        return expr;
    }

    fn parseTerm(self: *Parser) ParseResult!*ast.Expr {
        var expr = try self.parseFactor();
        while (self.match(.plus) or self.match(.minus)) {
            const op_token = self.previous();
            const right = try self.parseFactor();
            expr = try self.makeBinary(op_token.kind, expr, right);
        }
        return expr;
    }

    fn parseFactor(self: *Parser) ParseResult!*ast.Expr {
        var expr = try self.parsePrimary();
        while (self.match(.star) or self.match(.slash) or self.match(.percent)) {
            const op_token = self.previous();
            const right = try self.parsePrimary();
            expr = try self.makeBinary(op_token.kind, expr, right);
        }
        return expr;
    }

    fn parsePrimary(self: *Parser) ParseResult!*ast.Expr {
        if (self.match(.number)) return self.newExpr(.{ .number = self.previous().number.? });
        if (self.match(.kw_true)) return self.newExpr(.{ .boolean = true });
        if (self.match(.kw_false)) return self.newExpr(.{ .boolean = false });

        if (self.match(.identifier)) {
            const name = self.previous().lexeme;
            if (!self.match(.l_paren)) return self.newExpr(.{ .variable = name });

            var args = std.ArrayList(*ast.Expr).init(self.allocator);
            if (!self.current().is(.r_paren)) {
                while (true) {
                    try args.append(try self.parseExpression());
                    if (!self.match(.comma)) break;
                }
            }
            _ = try self.consume(.r_paren, error.ExpectedRightParen);
            return self.newExpr(.{ .call = .{ .name = name, .args = try args.toOwnedSlice() } });
        }

        if (self.match(.l_paren)) {
            const expr = try self.parseExpression();
            _ = try self.consume(.r_paren, error.ExpectedRightParen);
            return expr;
        }

        return error.ExpectedExpression;
    }

    fn makeBinary(self: *Parser, token_kind: TokenType, left: *ast.Expr, right: *ast.Expr) ParseResult!*ast.Expr {
        const op = switch (token_kind) {
            .plus => ast.BinaryOp.add,
            .minus => ast.BinaryOp.sub,
            .star => ast.BinaryOp.mul,
            .slash => ast.BinaryOp.div,
            .percent => ast.BinaryOp.mod,
            .equal_equal => ast.BinaryOp.eq,
            .bang_equal => ast.BinaryOp.neq,
            .less => ast.BinaryOp.lt,
            .less_equal => ast.BinaryOp.lte,
            .greater => ast.BinaryOp.gt,
            .greater_equal => ast.BinaryOp.gte,
            else => return error.InvalidBinaryOperator,
        };
        return self.newExpr(.{ .binary = .{ .op = op, .left = left, .right = right } });
    }

    fn newExpr(self: *Parser, expression: ast.Expr) ParseResult!*ast.Expr {
        const ptr = try self.allocator.create(ast.Expr);
        ptr.* = expression;
        return ptr;
    }

    fn consume(self: *Parser, kind: TokenType, parse_error: ParseError) ParseResult!Token {
        if (self.current().kind == kind) return self.advance();
        return parse_error;
    }

    fn match(self: *Parser, kind: TokenType) bool {
        if (self.current().kind != kind) return false;
        _ = self.advance();
        return true;
    }

    fn current(self: *const Parser) Token {
        return self.tokens[self.index];
    }

    fn peek(self: *const Parser) Token {
        if (self.index + 1 >= self.tokens.len) return self.tokens[self.index];
        return self.tokens[self.index + 1];
    }

    fn previous(self: *const Parser) Token {
        return self.tokens[self.index - 1];
    }

    fn advance(self: *Parser) Token {
        const token = self.tokens[self.index];
        self.index += 1;
        return token;
    }
};
