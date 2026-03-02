#pragma once

#include "lune/token.hpp"

#include <string>
#include <vector>

namespace lune {

class Lexer {
public:
    explicit Lexer(std::string source);
    std::vector<Token> tokenize();

private:
    [[nodiscard]] bool is_at_end() const;
    [[nodiscard]] char peek() const;
    [[nodiscard]] char peek_next() const;
    char advance();
    bool match(char expected);

    void skip_whitespace();
    Token make_token(TokenType type, std::string lexeme = {});
    Token identifier();
    Token number();
    Token string();

    std::string source_;
    std::size_t start_{0};
    std::size_t current_{0};
    std::size_t line_{1};
    std::size_t column_{1};
};

} // namespace lune
