#pragma once

#include <string>
#include <string_view>

namespace lune {

enum class TokenType {
    End,
    Identifier,
    Number,
    String,

    KwFn,
    KwConst,
    KwReturn,
    KwIf,
    KwElse,
    KwTrue,
    KwFalse,
    KwNull,

    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Dot,
    Colon,

    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Assign,
    ShortDecl,
    Arrow,

    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
};

struct Token {
    TokenType type{TokenType::End};
    std::string lexeme{};
    std::size_t line{1};
    std::size_t column{1};
};

std::string_view to_string(TokenType type);

} // namespace lune
