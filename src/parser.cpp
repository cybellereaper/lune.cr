#include "lune/parser.hpp"


namespace lune {

namespace {
template <typename T, typename... Args>
std::shared_ptr<T> make_node(Args&&... args) {
    return std::make_shared<T>(T{std::forward<Args>(args)...});
}

ExprPtr make_expr(Expr::Variant node) { return std::make_shared<Expr>(Expr{std::move(node)}); }
StmtPtr make_stmt(Stmt::Variant node) { return std::make_shared<Stmt>(Stmt{std::move(node)}); }
}

Parser::Parser(std::vector<Token> tokens) : tokens_(std::move(tokens)) {}

Program Parser::parse_program() {
    diagnostics_.clear();
    Program program;
    while (!is_at_end()) {
        program.items.push_back(declaration());
    }
    return program;
}

const std::vector<Diagnostic>& Parser::diagnostics() const { return diagnostics_; }

StmtPtr Parser::declaration() {
    if (match(TokenType::KwFn)) return function_declaration();
    if (match(TokenType::KwConst)) return const_declaration();
    return statement();
}

StmtPtr Parser::statement() {
    if (match(TokenType::LBrace)) return block_statement();
    if (match(TokenType::KwIf)) return if_statement();
    if (match(TokenType::KwReturn)) return return_statement();
    return simple_statement();
}

StmtPtr Parser::block_statement() {
    BlockStmt block;
    while (!check(TokenType::RBrace) && !is_at_end()) {
        block.statements.push_back(declaration());
    }
    consume(TokenType::RBrace, "Expected '}' at end of block");
    return make_stmt(block);
}

StmtPtr Parser::if_statement() {
    auto condition = expression();
    consume(TokenType::LBrace, "Expected '{' after if condition");
    auto then_branch = std::get<BlockStmt>(block_statement()->node);

    std::optional<BlockStmt> else_branch;
    if (match(TokenType::KwElse)) {
        consume(TokenType::LBrace, "Expected '{' after else");
        else_branch = std::get<BlockStmt>(block_statement()->node);
    }

    return make_stmt(IfStmt{.condition = condition, .then_branch = std::move(then_branch), .else_branch = std::move(else_branch)});
}

StmtPtr Parser::return_statement() {
    if (check(TokenType::RBrace) || check(TokenType::End)) {
        return make_stmt(ReturnStmt{.expr = std::nullopt});
    }
    return make_stmt(ReturnStmt{.expr = expression()});
}

StmtPtr Parser::function_declaration() {
    const auto& name = consume(TokenType::Identifier, "Expected function name").lexeme;
    consume(TokenType::LParen, "Expected '('");

    std::vector<std::string> params;
    if (!check(TokenType::RParen)) {
        do {
            params.push_back(consume(TokenType::Identifier, "Expected parameter name").lexeme);
        } while (match(TokenType::Comma));
    }
    consume(TokenType::RParen, "Expected ')' after parameters");
    consume(TokenType::LBrace, "Expected function body");
    auto body = std::get<BlockStmt>(block_statement()->node);
    return make_stmt(FunctionDecl{.name = name, .params = std::move(params), .body = std::move(body)});
}

StmtPtr Parser::const_declaration() {
    const auto& name = consume(TokenType::Identifier, "Expected const name").lexeme;
    consume(TokenType::Assign, "Expected '=' in const declaration");
    return make_stmt(ConstDeclStmt{.name = name, .expr = expression()});
}

StmtPtr Parser::simple_statement() {
    if (check(TokenType::Identifier) && tokens_[current_ + 1].type == TokenType::ShortDecl) {
        const auto name = advance().lexeme;
        advance();
        return make_stmt(ShortDeclStmt{.name = name, .expr = expression()});
    }
    if (check(TokenType::Identifier) && tokens_[current_ + 1].type == TokenType::Assign) {
        const auto name = advance().lexeme;
        advance();
        return make_stmt(AssignStmt{.name = name, .expr = expression()});
    }
    return make_stmt(ExprStmt{.expr = expression()});
}

ExprPtr Parser::expression() { return equality(); }

ExprPtr Parser::equality() {
    auto expr = comparison();
    while (match(TokenType::Eq) || match(TokenType::Ne)) {
        const auto op = previous().lexeme;
        expr = make_expr(BinaryExpr{.op = op, .lhs = expr, .rhs = comparison()});
    }
    return expr;
}

ExprPtr Parser::comparison() {
    auto expr = term();
    while (match(TokenType::Lt) || match(TokenType::Le) || match(TokenType::Gt) || match(TokenType::Ge)) {
        const auto op = previous().lexeme;
        expr = make_expr(BinaryExpr{.op = op, .lhs = expr, .rhs = term()});
    }
    return expr;
}

ExprPtr Parser::term() {
    auto expr = factor();
    while (match(TokenType::Plus) || match(TokenType::Minus)) {
        const auto op = previous().lexeme;
        expr = make_expr(BinaryExpr{.op = op, .lhs = expr, .rhs = factor()});
    }
    return expr;
}

ExprPtr Parser::factor() {
    auto expr = unary();
    while (match(TokenType::Star) || match(TokenType::Slash) || match(TokenType::Percent)) {
        const auto op = previous().lexeme;
        expr = make_expr(BinaryExpr{.op = op, .lhs = expr, .rhs = unary()});
    }
    return expr;
}

ExprPtr Parser::unary() {
    if (match(TokenType::Minus)) {
        return make_expr(BinaryExpr{.op = "-", .lhs = make_expr(NumberExpr{0.0}), .rhs = unary()});
    }
    return call();
}

ExprPtr Parser::call() {
    auto expr = primary();
    while (match(TokenType::LParen)) {
        if (!std::holds_alternative<IdentifierExpr>(expr->node)) {
            add_error_here("Only named calls are currently supported");
            return expr;
        }
        CallExpr call_expr{.callee = std::get<IdentifierExpr>(expr->node).name};
        if (!check(TokenType::RParen)) {
            do {
                call_expr.args.push_back(expression());
            } while (match(TokenType::Comma));
        }
        consume(TokenType::RParen, "Expected ')' after call args");
        expr = make_expr(std::move(call_expr));
    }
    return expr;
}

ExprPtr Parser::primary() {
    if (match(TokenType::Number)) return make_expr(NumberExpr{std::stod(previous().lexeme)});
    if (match(TokenType::String)) return make_expr(StringExpr{previous().lexeme});
    if (match(TokenType::KwTrue)) return make_expr(BoolExpr{true});
    if (match(TokenType::KwFalse)) return make_expr(BoolExpr{false});
    if (match(TokenType::KwNull)) return make_expr(NullExpr{});
    if (match(TokenType::Identifier)) return make_expr(IdentifierExpr{previous().lexeme});
    if (match(TokenType::LParen)) {
        auto expr = expression();
        consume(TokenType::RParen, "Expected ')' after expression");
        return expr;
    }
    add_error_here("Expected expression");
    return make_expr(NullExpr{});
}


bool Parser::is_at_end() const { return peek().type == TokenType::End; }
const Token& Parser::peek() const { return tokens_[current_]; }
const Token& Parser::previous() const { return tokens_[current_ - 1]; }
const Token& Parser::advance() {
    if (!is_at_end()) ++current_;
    return previous();
}
bool Parser::check(TokenType type) const { return !is_at_end() && peek().type == type; }
bool Parser::match(TokenType type) {
    if (!check(type)) return false;
    advance();
    return true;
}
const Token& Parser::consume(TokenType type, const char* error_message) {
    if (check(type)) return advance();
    add_error_here(error_message);
    return peek();
}

void Parser::add_error_here(const char* error_message) {
    diagnostics_.push_back(Diagnostic{.message = error_message, .line = peek().line, .column = peek().column});
}

} // namespace lune
