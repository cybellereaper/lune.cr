#pragma once

#include "lune/ast.hpp"
#include "lune/token.hpp"

#include <vector>

namespace lune {

class Parser {
public:
    explicit Parser(std::vector<Token> tokens);
    Program parse_program();
    [[nodiscard]] const std::vector<Diagnostic>& diagnostics() const;

private:
    [[nodiscard]] bool is_at_end() const;
    [[nodiscard]] const Token& peek() const;
    [[nodiscard]] const Token& previous() const;
    const Token& advance();
    bool check(TokenType type) const;
    bool check_next(TokenType type) const;
    bool match(TokenType type);
    const Token& consume(TokenType type, const char* error_message);
    void add_error_here(const char* error_message);
    void synchronize();

    StmtPtr declaration();
    StmtPtr statement();
    StmtPtr block_statement();
    StmtPtr if_statement();
    StmtPtr while_statement();
    StmtPtr return_statement();
    StmtPtr function_declaration();
    StmtPtr const_declaration();
    StmtPtr simple_statement();

    ExprPtr expression();
    ExprPtr equality();
    ExprPtr comparison();
    ExprPtr term();
    ExprPtr factor();
    ExprPtr unary();
    ExprPtr call();
    ExprPtr primary();

    std::vector<Token> tokens_;
    std::size_t current_{0};
    std::vector<Diagnostic> diagnostics_{};
};

} // namespace lune
