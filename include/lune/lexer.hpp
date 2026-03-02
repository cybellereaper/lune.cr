#pragma once

#include "lune/token.hpp"

#include <string>
#include <vector>

namespace lune {

class Lexer {
public:
    explicit Lexer(std::string source);
    std::vector<Token> tokenize();
    [[nodiscard]] const std::vector<Diagnostic>& diagnostics() const;

private:
    [[nodiscard]] bool is_at_end() const;
    [[nodiscard]] char peek() const;
    [[nodiscard]] char peek_next() const;
    char advance();
    bool match(char expected);

    std::string collect_trivia();
    Token make_token(TokenType type, std::string lexeme = {});
    Token identifier(char first);
    Token number(char first);
    Token string();

    std::string source_;
    std::size_t start_{0};
    std::size_t current_{0};
    std::size_t line_{1};
    std::size_t column_{1};
    std::string pending_trivia_{};
    std::vector<Diagnostic> diagnostics_{};
};

} // namespace lune
