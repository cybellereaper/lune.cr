#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenType {
    End,
    Identifier,
    Number,
    String,
    KwFn,
    KwIf,
    KwElse,
    KwWhile,
    KwConst,
    KwReturn,
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
}

pub const TOKEN_TYPE_NAMES: &[(TokenType, &str)] = &[
    (TokenType::End, "end"),
    (TokenType::Identifier, "identifier"),
    (TokenType::Number, "number"),
    (TokenType::String, "string"),
    (TokenType::KwFn, "fn"),
    (TokenType::KwIf, "if"),
    (TokenType::KwElse, "else"),
    (TokenType::KwWhile, "while"),
    (TokenType::KwConst, "const"),
    (TokenType::KwReturn, "return"),
    (TokenType::KwTrue, "true"),
    (TokenType::KwFalse, "false"),
    (TokenType::KwNull, "null"),
    (TokenType::LParen, "("),
    (TokenType::RParen, ")"),
    (TokenType::LBrace, "{"),
    (TokenType::RBrace, "}"),
    (TokenType::Comma, ","),
    (TokenType::Dot, "."),
    (TokenType::Colon, ":"),
    (TokenType::Plus, "+"),
    (TokenType::Minus, "-"),
    (TokenType::Star, "*"),
    (TokenType::Slash, "/"),
    (TokenType::Percent, "%"),
    (TokenType::Assign, "="),
    (TokenType::ShortDecl, ":="),
    (TokenType::Arrow, "=>"),
    (TokenType::Eq, "=="),
    (TokenType::Ne, "!="),
    (TokenType::Lt, "<"),
    (TokenType::Le, "<="),
    (TokenType::Gt, ">"),
    (TokenType::Ge, ">="),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub leading_trivia: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticKind {
    UnexpectedTokenBang,
    UnexpectedCharacter,
    UnterminatedString,
}

impl DiagnosticKind {
    pub fn message(self) -> &'static str {
        match self {
            DiagnosticKind::UnexpectedTokenBang => "Unexpected token !",
            DiagnosticKind::UnexpectedCharacter => "Unexpected character in input",
            DiagnosticKind::UnterminatedString => "Unterminated string",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub kind: DiagnosticKind,
    pub line: usize,
    pub column: usize,
}

impl Diagnostic {
    pub fn message(&self) -> &'static str {
        self.kind.message()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexerResult {
    pub tokens: Vec<Token>,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, Copy)]
struct TokenStart {
    offset: usize,
    line: usize,
    column: usize,
}

struct Scanner<'a> {
    source: &'a str,
    offset: usize,
    line: usize,
    column: usize,
}

impl<'a> Scanner<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            offset: 0,
            line: 1,
            column: 1,
        }
    }

    fn at_end(&self) -> bool {
        self.offset >= self.source.len()
    }

    fn peek(&self) -> Option<u8> {
        self.source.as_bytes().get(self.offset).copied()
    }

    fn peek_next(&self) -> Option<u8> {
        self.source.as_bytes().get(self.offset + 1).copied()
    }

    fn advance(&mut self) -> Option<u8> {
        let b = self.peek()?;
        self.offset += 1;
        if b == b'\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(b)
    }

    fn matches(&mut self, expected: u8) -> bool {
        if self.peek() == Some(expected) {
            self.advance();
            true
        } else {
            false
        }
    }
}

pub struct Lexer<'a> {
    scanner: Scanner<'a>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            scanner: Scanner::new(source),
        }
    }

    pub fn tokenize(&mut self) -> LexerResult {
        let mut tokens = Vec::new();
        let mut diagnostics = Vec::new();

        while !self.scanner.at_end() {
            let trivia_start = self.scanner.offset;
            self.skip_trivia();
            let trivia = self.slice(trivia_start, self.scanner.offset).to_owned();

            if self.scanner.at_end() {
                break;
            }

            let start = TokenStart {
                offset: self.scanner.offset,
                line: self.scanner.line,
                column: self.scanner.column,
            };

            let Some(current) = self.scanner.advance() else {
                continue;
            };

            self.scan_token(&mut tokens, &mut diagnostics, start, trivia, current);
        }

        tokens.push(Token {
            token_type: TokenType::End,
            lexeme: String::new(),
            leading_trivia: String::new(),
            line: self.scanner.line,
            column: self.scanner.column,
        });

        LexerResult {
            tokens,
            diagnostics,
        }
    }

    fn scan_token(
        &mut self,
        tokens: &mut Vec<Token>,
        diagnostics: &mut Vec<Diagnostic>,
        start: TokenStart,
        trivia: String,
        current: u8,
    ) {
        if let Some(token_type) = single_char_token(current) {
            self.append_token(tokens, token_type, trivia, start, self.scanner.offset);
            return;
        }

        match current {
            b':' => {
                if self.scanner.matches(b'=') {
                    self.append_token(
                        tokens,
                        TokenType::ShortDecl,
                        trivia,
                        start,
                        self.scanner.offset,
                    );
                } else {
                    self.append_token(tokens, TokenType::Colon, trivia, start, self.scanner.offset);
                }
            }
            b'=' => {
                if self.scanner.matches(b'=') {
                    self.append_token(tokens, TokenType::Eq, trivia, start, self.scanner.offset);
                } else if self.scanner.matches(b'>') {
                    self.append_token(tokens, TokenType::Arrow, trivia, start, self.scanner.offset);
                } else {
                    self.append_token(
                        tokens,
                        TokenType::Assign,
                        trivia,
                        start,
                        self.scanner.offset,
                    );
                }
            }
            b'!' => {
                if self.scanner.matches(b'=') {
                    self.append_token(tokens, TokenType::Ne, trivia, start, self.scanner.offset);
                } else {
                    self.append_diagnostic(diagnostics, DiagnosticKind::UnexpectedTokenBang, start);
                }
            }
            b'<' => {
                if self.scanner.matches(b'=') {
                    self.append_token(tokens, TokenType::Le, trivia, start, self.scanner.offset);
                } else {
                    self.append_token(tokens, TokenType::Lt, trivia, start, self.scanner.offset);
                }
            }
            b'>' => {
                if self.scanner.matches(b'=') {
                    self.append_token(tokens, TokenType::Ge, trivia, start, self.scanner.offset);
                } else {
                    self.append_token(tokens, TokenType::Gt, trivia, start, self.scanner.offset);
                }
            }
            b'"' => self.scan_string(tokens, diagnostics, start, trivia),
            _ if is_ascii_digit(current) => self.scan_number(tokens, start, trivia),
            _ if identifier_start(current) => self.scan_identifier(tokens, start, trivia),
            _ => self.append_diagnostic(diagnostics, DiagnosticKind::UnexpectedCharacter, start),
        }
    }

    fn scan_identifier(&mut self, tokens: &mut Vec<Token>, start: TokenStart, trivia: String) {
        while let Some(next_char) = self.scanner.peek() {
            if !identifier_part(next_char) {
                break;
            }
            self.scanner.advance();
        }

        let lexeme = self.slice(start.offset, self.scanner.offset);
        let token_type = keyword_token(lexeme).unwrap_or(TokenType::Identifier);
        self.append_token(tokens, token_type, trivia, start, self.scanner.offset);
    }

    fn scan_number(&mut self, tokens: &mut Vec<Token>, start: TokenStart, trivia: String) {
        while let Some(next_char) = self.scanner.peek() {
            if !is_ascii_digit(next_char) {
                break;
            }
            self.scanner.advance();
        }

        if self.scanner.peek() == Some(b'.') && self.scanner.peek_next().is_some_and(is_ascii_digit)
        {
            self.scanner.advance();
            while let Some(next_char) = self.scanner.peek() {
                if !is_ascii_digit(next_char) {
                    break;
                }
                self.scanner.advance();
            }
        }

        self.append_token(
            tokens,
            TokenType::Number,
            trivia,
            start,
            self.scanner.offset,
        );
    }

    fn scan_string(
        &mut self,
        tokens: &mut Vec<Token>,
        diagnostics: &mut Vec<Diagnostic>,
        start: TokenStart,
        trivia: String,
    ) {
        let string_content_start = self.scanner.offset;

        while let Some(next_char) = self.scanner.peek() {
            if next_char == b'"' {
                break;
            }
            self.scanner.advance();
        }

        if self.scanner.at_end() {
            self.append_token_slice(
                tokens,
                TokenType::String,
                trivia,
                start,
                string_content_start,
                self.scanner.offset,
            );
            self.append_diagnostic(diagnostics, DiagnosticKind::UnterminatedString, start);
            return;
        }

        let string_content_end = self.scanner.offset;
        self.scanner.advance();
        self.append_token_slice(
            tokens,
            TokenType::String,
            trivia,
            start,
            string_content_start,
            string_content_end,
        );
    }

    fn skip_trivia(&mut self) {
        while !self.scanner.at_end() {
            match self.scanner.peek() {
                Some(ch) if is_ascii_whitespace(ch) => {
                    self.scanner.advance();
                }
                Some(b'/') if self.scanner.peek_next() == Some(b'/') => {
                    self.scanner.advance();
                    self.scanner.advance();
                    while let Some(next_char) = self.scanner.peek() {
                        if next_char == b'\n' {
                            break;
                        }
                        self.scanner.advance();
                    }
                }
                _ => break,
            }
        }
    }

    fn append_token(
        &self,
        tokens: &mut Vec<Token>,
        token_type: TokenType,
        trivia: String,
        start: TokenStart,
        end_offset: usize,
    ) {
        self.append_token_slice(tokens, token_type, trivia, start, start.offset, end_offset);
    }

    fn append_token_slice(
        &self,
        tokens: &mut Vec<Token>,
        token_type: TokenType,
        trivia: String,
        start: TokenStart,
        start_offset: usize,
        end_offset: usize,
    ) {
        tokens.push(Token {
            token_type,
            lexeme: self.slice(start_offset, end_offset).to_owned(),
            leading_trivia: trivia,
            line: start.line,
            column: start.column,
        });
    }

    fn append_diagnostic(
        &self,
        diagnostics: &mut Vec<Diagnostic>,
        kind: DiagnosticKind,
        start: TokenStart,
    ) {
        diagnostics.push(Diagnostic {
            kind,
            line: start.line,
            column: start.column,
        });
    }

    fn slice(&self, start: usize, end: usize) -> &str {
        &self.scanner.source[start..end]
    }
}

fn single_char_token(ch: u8) -> Option<TokenType> {
    match ch {
        b'(' => Some(TokenType::LParen),
        b')' => Some(TokenType::RParen),
        b'{' => Some(TokenType::LBrace),
        b'}' => Some(TokenType::RBrace),
        b',' => Some(TokenType::Comma),
        b'.' => Some(TokenType::Dot),
        b'+' => Some(TokenType::Plus),
        b'-' => Some(TokenType::Minus),
        b'*' => Some(TokenType::Star),
        b'/' => Some(TokenType::Slash),
        b'%' => Some(TokenType::Percent),
        _ => None,
    }
}

fn is_ascii_digit(ch: u8) -> bool {
    ch.is_ascii_digit()
}

fn is_ascii_whitespace(ch: u8) -> bool {
    ch.is_ascii_whitespace()
}

fn identifier_start(ch: u8) -> bool {
    ch.is_ascii_alphabetic() || ch == b'_'
}

fn identifier_part(ch: u8) -> bool {
    identifier_start(ch) || is_ascii_digit(ch)
}

fn keyword_token(lexeme: &str) -> Option<TokenType> {
    match lexeme {
        "fn" => Some(TokenType::KwFn),
        "if" => Some(TokenType::KwIf),
        "else" => Some(TokenType::KwElse),
        "while" => Some(TokenType::KwWhile),
        "const" => Some(TokenType::KwConst),
        "return" => Some(TokenType::KwReturn),
        "true" => Some(TokenType::KwTrue),
        "false" => Some(TokenType::KwFalse),
        "null" => Some(TokenType::KwNull),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizes_keywords_operators_numbers_and_trivia() {
        let mut lexer = Lexer::new("  const value := 42.5 // note\nreturn value");
        let result = lexer.tokenize();

        assert_eq!(result.tokens.len(), 7);
        assert_eq!(result.tokens[0].token_type, TokenType::KwConst);
        assert_eq!(result.tokens[0].leading_trivia, "  ");
        assert_eq!(result.tokens[0].lexeme, "const");
        assert_eq!(result.tokens[1].token_type, TokenType::Identifier);
        assert_eq!(result.tokens[1].lexeme, "value");
        assert_eq!(result.tokens[2].token_type, TokenType::ShortDecl);
        assert_eq!(result.tokens[2].lexeme, ":=");
        assert_eq!(result.tokens[3].token_type, TokenType::Number);
        assert_eq!(result.tokens[3].lexeme, "42.5");
        assert_eq!(result.tokens[4].token_type, TokenType::KwReturn);
        assert_eq!(result.tokens[4].leading_trivia, " // note\n");
        assert_eq!(result.tokens[5].token_type, TokenType::Identifier);
        assert_eq!(result.tokens[5].lexeme, "value");
        assert_eq!(result.tokens[6].token_type, TokenType::End);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn reports_unexpected_bang_token() {
        let mut lexer = Lexer::new("!");
        let result = lexer.tokenize();

        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(
            result.diagnostics[0].kind,
            DiagnosticKind::UnexpectedTokenBang
        );
        assert_eq!(result.diagnostics[0].message(), "Unexpected token !");
        assert_eq!(result.tokens.len(), 1);
        assert_eq!(result.tokens[0].token_type, TokenType::End);
    }

    #[test]
    fn reports_unterminated_string_and_keeps_parsed_text() {
        let mut lexer = Lexer::new("\"hello");
        let result = lexer.tokenize();

        assert_eq!(result.tokens.len(), 2);
        assert_eq!(result.tokens[0].token_type, TokenType::String);
        assert_eq!(result.tokens[0].lexeme, "hello");
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(
            result.diagnostics[0].kind,
            DiagnosticKind::UnterminatedString
        );
        assert_eq!(result.diagnostics[0].message(), "Unterminated string");
    }

    #[test]
    fn parses_terminated_string_without_diagnostics() {
        let mut lexer = Lexer::new("\"hello\"");
        let result = lexer.tokenize();

        assert_eq!(result.tokens.len(), 2);
        assert_eq!(result.tokens[0].token_type, TokenType::String);
        assert_eq!(result.tokens[0].lexeme, "hello");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn parses_comparison_and_arrow_operators() {
        let mut lexer = Lexer::new("a==b != c <= d >= e => f");
        let result = lexer.tokenize();

        let expected = vec![
            TokenType::Identifier,
            TokenType::Eq,
            TokenType::Identifier,
            TokenType::Ne,
            TokenType::Identifier,
            TokenType::Le,
            TokenType::Identifier,
            TokenType::Ge,
            TokenType::Identifier,
            TokenType::Arrow,
            TokenType::Identifier,
            TokenType::End,
        ];

        let got: Vec<TokenType> = result.tokens.iter().map(|t| t.token_type).collect();
        assert_eq!(got, expected);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn captures_unknown_characters_as_diagnostics_and_still_emits_end_token() {
        let mut lexer = Lexer::new("@");
        let result = lexer.tokenize();

        assert_eq!(result.tokens.len(), 1);
        assert_eq!(result.tokens[0].token_type, TokenType::End);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(
            result.diagnostics[0].kind,
            DiagnosticKind::UnexpectedCharacter
        );
    }

    #[test]
    fn supports_integer_without_fraction() {
        let mut lexer = Lexer::new("42.");
        let result = lexer.tokenize();

        assert_eq!(result.tokens[0].token_type, TokenType::Number);
        assert_eq!(result.tokens[0].lexeme, "42");
        assert_eq!(result.tokens[1].token_type, TokenType::Dot);
    }
}
