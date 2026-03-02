#include "lune/lexer.hpp"

#include <cctype>
#include <stdexcept>
#include <unordered_map>

namespace lune {

namespace {
const std::unordered_map<std::string, TokenType> kKeywords{
    {"fn", TokenType::KwFn},       {"const", TokenType::KwConst},
    {"return", TokenType::KwReturn}, {"if", TokenType::KwIf},
    {"else", TokenType::KwElse},   {"true", TokenType::KwTrue},
    {"false", TokenType::KwFalse}, {"null", TokenType::KwNull},
};
}

std::string_view to_string(TokenType type) {
    switch (type) {
    case TokenType::End: return "end";
    case TokenType::Identifier: return "identifier";
    case TokenType::Number: return "number";
    case TokenType::String: return "string";
    case TokenType::KwFn: return "fn";
    case TokenType::KwConst: return "const";
    case TokenType::KwReturn: return "return";
    case TokenType::KwIf: return "if";
    case TokenType::KwElse: return "else";
    case TokenType::KwTrue: return "true";
    case TokenType::KwFalse: return "false";
    case TokenType::KwNull: return "null";
    case TokenType::LParen: return "(";
    case TokenType::RParen: return ")";
    case TokenType::LBrace: return "{";
    case TokenType::RBrace: return "}";
    case TokenType::Comma: return ",";
    case TokenType::Dot: return ".";
    case TokenType::Colon: return ":";
    case TokenType::Plus: return "+";
    case TokenType::Minus: return "-";
    case TokenType::Star: return "*";
    case TokenType::Slash: return "/";
    case TokenType::Percent: return "%";
    case TokenType::Assign: return "=";
    case TokenType::ShortDecl: return ":=";
    case TokenType::Arrow: return "=>";
    case TokenType::Eq: return "==";
    case TokenType::Ne: return "!=";
    case TokenType::Lt: return "<";
    case TokenType::Le: return "<=";
    case TokenType::Gt: return ">";
    case TokenType::Ge: return ">=";
    }
    return "unknown";
}

Lexer::Lexer(std::string source) : source_(std::move(source)) {}

std::vector<Token> Lexer::tokenize() {
    std::vector<Token> tokens;
    while (!is_at_end()) {
        skip_whitespace();
        if (is_at_end()) {
            break;
        }
        start_ = current_;
        const char c = advance();
        switch (c) {
        case '(': tokens.push_back(make_token(TokenType::LParen, "(")); break;
        case ')': tokens.push_back(make_token(TokenType::RParen, ")")); break;
        case '{': tokens.push_back(make_token(TokenType::LBrace, "{")); break;
        case '}': tokens.push_back(make_token(TokenType::RBrace, "}")); break;
        case ',': tokens.push_back(make_token(TokenType::Comma, ",")); break;
        case '.': tokens.push_back(make_token(TokenType::Dot, ".")); break;
        case '+': tokens.push_back(make_token(TokenType::Plus, "+")); break;
        case '-': tokens.push_back(make_token(TokenType::Minus, "-")); break;
        case '*': tokens.push_back(make_token(TokenType::Star, "*")); break;
        case '/': tokens.push_back(make_token(TokenType::Slash, "/")); break;
        case '%': tokens.push_back(make_token(TokenType::Percent, "%")); break;
        case ':':
            if (match('=')) {
                tokens.push_back(make_token(TokenType::ShortDecl, ":="));
            } else {
                tokens.push_back(make_token(TokenType::Colon, ":"));
            }
            break;
        case '=':
            if (match('=')) {
                tokens.push_back(make_token(TokenType::Eq, "=="));
            } else if (match('>')) {
                tokens.push_back(make_token(TokenType::Arrow, "=>"));
            } else {
                tokens.push_back(make_token(TokenType::Assign, "="));
            }
            break;
        case '!':
            if (!match('=')) {
                throw std::runtime_error("Unexpected token !");
            }
            tokens.push_back(make_token(TokenType::Ne, "!="));
            break;
        case '<':
            if (match('=')) {
                tokens.push_back(make_token(TokenType::Le, "<="));
            } else {
                tokens.push_back(make_token(TokenType::Lt, "<"));
            }
            break;
        case '>':
            if (match('=')) {
                tokens.push_back(make_token(TokenType::Ge, ">="));
            } else {
                tokens.push_back(make_token(TokenType::Gt, ">"));
            }
            break;
        case '"': tokens.push_back(string()); break;
        default:
            if (std::isdigit(static_cast<unsigned char>(c))) {
                --current_;
                --column_;
                tokens.push_back(number());
            } else if (std::isalpha(static_cast<unsigned char>(c)) || c == '_') {
                --current_;
                --column_;
                tokens.push_back(identifier());
            } else {
                throw std::runtime_error("Unexpected character in input");
            }
            break;
        }
    }
    tokens.push_back(make_token(TokenType::End, ""));
    return tokens;
}

bool Lexer::is_at_end() const { return current_ >= source_.size(); }
char Lexer::peek() const { return is_at_end() ? '\0' : source_[current_]; }
char Lexer::peek_next() const { return current_ + 1 < source_.size() ? source_[current_ + 1] : '\0'; }

char Lexer::advance() {
    if (is_at_end()) {
        return '\0';
    }
    const char c = source_[current_++];
    if (c == '\n') {
        ++line_;
        column_ = 1;
    } else {
        ++column_;
    }
    return c;
}

bool Lexer::match(char expected) {
    if (is_at_end() || source_[current_] != expected) {
        return false;
    }
    ++current_;
    ++column_;
    return true;
}

void Lexer::skip_whitespace() {
    while (!is_at_end()) {
        const char c = peek();
        if (std::isspace(static_cast<unsigned char>(c))) {
            advance();
            continue;
        }
        if (c == '/' && peek_next() == '/') {
            while (!is_at_end() && peek() != '\n') {
                advance();
            }
            continue;
        }
        break;
    }
}

Token Lexer::make_token(TokenType type, std::string lexeme) {
    return Token{.type = type, .lexeme = lexeme.empty() ? source_.substr(start_, current_ - start_) : std::move(lexeme), .line = line_, .column = column_};
}

Token Lexer::identifier() {
    start_ = current_;
    while (std::isalnum(static_cast<unsigned char>(peek())) || peek() == '_') {
        advance();
    }
    const auto lexeme = source_.substr(start_, current_ - start_);
    const auto it = kKeywords.find(lexeme);
    return make_token(it == kKeywords.end() ? TokenType::Identifier : it->second, lexeme);
}

Token Lexer::number() {
    start_ = current_;
    while (std::isdigit(static_cast<unsigned char>(peek()))) {
        advance();
    }
    if (peek() == '.' && std::isdigit(static_cast<unsigned char>(peek_next()))) {
        advance();
        while (std::isdigit(static_cast<unsigned char>(peek()))) {
            advance();
        }
    }
    return make_token(TokenType::Number, source_.substr(start_, current_ - start_));
}

Token Lexer::string() {
    start_ = current_;
    while (!is_at_end() && peek() != '"') {
        advance();
    }
    if (is_at_end()) {
        throw std::runtime_error("Unterminated string");
    }
    const auto value = source_.substr(start_, current_ - start_);
    advance();
    return make_token(TokenType::String, value);
}

} // namespace lune
