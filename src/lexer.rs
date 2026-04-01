#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl TokenType {
    pub fn display_name(self) -> &'static str {
        match self {
            TokenType::End => "end",
            TokenType::Identifier => "identifier",
            TokenType::Number => "number",
            TokenType::String => "string",
            TokenType::KwFn => "fn",
            TokenType::KwIf => "if",
            TokenType::KwElse => "else",
            TokenType::KwWhile => "while",
            TokenType::KwConst => "const",
            TokenType::KwReturn => "return",
            TokenType::KwTrue => "true",
            TokenType::KwFalse => "false",
            TokenType::KwNull => "null",
            TokenType::LParen => "(",
            TokenType::RParen => ")",
            TokenType::LBrace => "{",
            TokenType::RBrace => "}",
            TokenType::Comma => ",",
            TokenType::Dot => ".",
            TokenType::Colon => ":",
            TokenType::Plus => "+",
            TokenType::Minus => "-",
            TokenType::Star => "*",
            TokenType::Slash => "/",
            TokenType::Percent => "%",
            TokenType::Assign => "=",
            TokenType::ShortDecl => ":=",
            TokenType::Arrow => "=>",
            TokenType::Eq => "==",
            TokenType::Ne => "!=",
            TokenType::Lt => "<",
            TokenType::Le => "<=",
            TokenType::Gt => ">",
            TokenType::Ge => ">=",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token<'src> {
    pub token_type: TokenType,
    pub lexeme: &'src str,
    pub leading_trivia: &'src str,
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

#[derive(Debug)]
pub struct LexResult<'src> {
    pub tokens: Vec<Token<'src>>,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, Copy)]
struct TokenStart {
    offset: usize,
    line: usize,
    column: usize,
}

#[derive(Debug)]
struct Scanner<'src> {
    source: &'src str,
    offset: usize,
    line: usize,
    column: usize,
}

impl<'src> Scanner<'src> {
    fn new(source: &'src str) -> Self {
        Self {
            source,
            offset: 0,
            line: 1,
            column: 1,
        }
    }

    fn is_at_end(&self) -> bool {
        self.offset >= self.source.len()
    }

    fn peek(&self) -> Option<u8> {
        self.source.as_bytes().get(self.offset).copied()
    }

    fn peek_next(&self) -> Option<u8> {
        self.source.as_bytes().get(self.offset + 1).copied()
    }

    fn advance(&mut self) -> Option<u8> {
        let current = self.peek()?;
        self.offset += 1;

        if current == b'\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }

        Some(current)
    }

    fn consume_if_matches(&mut self, expected: u8) -> bool {
        if self.peek() != Some(expected) {
            return false;
        }
        self.advance();
        true
    }
}

pub struct Lexer<'src> {
    scanner: Scanner<'src>,
}

impl<'src> Lexer<'src> {
    pub fn new(source: &'src str) -> Self {
        Self {
            scanner: Scanner::new(source),
        }
    }

    pub fn tokenize(&mut self) -> LexResult<'src> {
        let mut result = LexResult {
            tokens: Vec::new(),
            diagnostics: Vec::new(),
        };

        while !self.scanner.is_at_end() {
            let trivia_start = self.scanner.offset;
            self.skip_trivia();
            let trivia = &self.scanner.source[trivia_start..self.scanner.offset];

            if self.scanner.is_at_end() {
                break;
            }

            let token_start = TokenStart {
                offset: self.scanner.offset,
                line: self.scanner.line,
                column: self.scanner.column,
            };

            let current = self.scanner.advance().expect("checked end-of-input");
            self.scan_token(&mut result, token_start, trivia, current);
        }

        result.tokens.push(Token {
            token_type: TokenType::End,
            lexeme: "",
            leading_trivia: "",
            line: self.scanner.line,
            column: self.scanner.column,
        });

        result
    }

    fn scan_token(
        &mut self,
        result: &mut LexResult<'src>,
        start: TokenStart,
        trivia: &'src str,
        current: u8,
    ) {
        if let Some(token_type) = single_character_token(current) {
            self.append_token(result, token_type, trivia, start, self.scanner.offset);
            return;
        }

        match current {
            b':' => {
                if self.scanner.consume_if_matches(b'=') {
                    self.append_token(
                        result,
                        TokenType::ShortDecl,
                        trivia,
                        start,
                        self.scanner.offset,
                    );
                } else {
                    self.append_token(result, TokenType::Colon, trivia, start, self.scanner.offset);
                }
            }
            b'=' => {
                if self.scanner.consume_if_matches(b'=') {
                    self.append_token(result, TokenType::Eq, trivia, start, self.scanner.offset);
                } else if self.scanner.consume_if_matches(b'>') {
                    self.append_token(result, TokenType::Arrow, trivia, start, self.scanner.offset);
                } else {
                    self.append_token(
                        result,
                        TokenType::Assign,
                        trivia,
                        start,
                        self.scanner.offset,
                    );
                }
            }
            b'!' => {
                if self.scanner.consume_if_matches(b'=') {
                    self.append_token(result, TokenType::Ne, trivia, start, self.scanner.offset);
                } else {
                    self.append_diagnostic(result, DiagnosticKind::UnexpectedTokenBang, start);
                }
            }
            b'<' => {
                let token_type = if self.scanner.consume_if_matches(b'=') {
                    TokenType::Le
                } else {
                    TokenType::Lt
                };
                self.append_token(result, token_type, trivia, start, self.scanner.offset);
            }
            b'>' => {
                let token_type = if self.scanner.consume_if_matches(b'=') {
                    TokenType::Ge
                } else {
                    TokenType::Gt
                };
                self.append_token(result, token_type, trivia, start, self.scanner.offset);
            }
            b'"' => self.scan_string(result, start, trivia),
            _ if current.is_ascii_digit() => self.scan_number(result, start, trivia),
            _ if is_identifier_start(current) => self.scan_identifier(result, start, trivia),
            _ => self.append_diagnostic(result, DiagnosticKind::UnexpectedCharacter, start),
        }
    }

    fn scan_identifier(
        &mut self,
        result: &mut LexResult<'src>,
        start: TokenStart,
        trivia: &'src str,
    ) {
        while matches!(self.scanner.peek(), Some(next) if is_identifier_part(next)) {
            self.scanner.advance();
        }

        let lexeme = &self.scanner.source[start.offset..self.scanner.offset];
        result.tokens.push(Token {
            token_type: keyword_type(lexeme),
            lexeme,
            leading_trivia: trivia,
            line: start.line,
            column: start.column,
        });
    }

    fn scan_number(&mut self, result: &mut LexResult<'src>, start: TokenStart, trivia: &'src str) {
        while matches!(self.scanner.peek(), Some(next) if next.is_ascii_digit()) {
            self.scanner.advance();
        }

        if self.scanner.peek() == Some(b'.')
            && matches!(self.scanner.peek_next(), Some(next) if next.is_ascii_digit())
        {
            self.scanner.advance();
            while matches!(self.scanner.peek(), Some(next) if next.is_ascii_digit()) {
                self.scanner.advance();
            }
        }

        self.append_token(
            result,
            TokenType::Number,
            trivia,
            start,
            self.scanner.offset,
        );
    }

    fn scan_string(&mut self, result: &mut LexResult<'src>, start: TokenStart, trivia: &'src str) {
        let string_content_start = self.scanner.offset;

        while !self.scanner.is_at_end() && self.scanner.peek() != Some(b'"') {
            self.scanner.advance();
        }

        if self.scanner.is_at_end() {
            result.tokens.push(Token {
                token_type: TokenType::String,
                lexeme: &self.scanner.source[string_content_start..self.scanner.offset],
                leading_trivia: trivia,
                line: start.line,
                column: start.column,
            });
            self.append_diagnostic(result, DiagnosticKind::UnterminatedString, start);
            return;
        }

        let string_content_end = self.scanner.offset;
        self.scanner.advance();

        result.tokens.push(Token {
            token_type: TokenType::String,
            lexeme: &self.scanner.source[string_content_start..string_content_end],
            leading_trivia: trivia,
            line: start.line,
            column: start.column,
        });
    }

    fn skip_trivia(&mut self) {
        while let Some(current) = self.scanner.peek() {
            if current.is_ascii_whitespace() {
                self.scanner.advance();
                continue;
            }

            if current == b'/' && self.scanner.peek_next() == Some(b'/') {
                self.scanner.advance();
                self.scanner.advance();
                while !self.scanner.is_at_end() && self.scanner.peek() != Some(b'\n') {
                    self.scanner.advance();
                }
                continue;
            }

            break;
        }
    }

    fn append_token(
        &self,
        result: &mut LexResult<'src>,
        token_type: TokenType,
        trivia: &'src str,
        start: TokenStart,
        end_offset: usize,
    ) {
        result.tokens.push(Token {
            token_type,
            lexeme: &self.scanner.source[start.offset..end_offset],
            leading_trivia: trivia,
            line: start.line,
            column: start.column,
        });
    }

    fn append_diagnostic(
        &self,
        result: &mut LexResult<'src>,
        kind: DiagnosticKind,
        start: TokenStart,
    ) {
        result.diagnostics.push(Diagnostic {
            kind,
            line: start.line,
            column: start.column,
        });
    }
}

fn single_character_token(char: u8) -> Option<TokenType> {
    Some(match char {
        b'(' => TokenType::LParen,
        b')' => TokenType::RParen,
        b'{' => TokenType::LBrace,
        b'}' => TokenType::RBrace,
        b',' => TokenType::Comma,
        b'.' => TokenType::Dot,
        b'+' => TokenType::Plus,
        b'-' => TokenType::Minus,
        b'*' => TokenType::Star,
        b'/' => TokenType::Slash,
        b'%' => TokenType::Percent,
        _ => return None,
    })
}

fn is_identifier_start(char: u8) -> bool {
    char.is_ascii_alphabetic() || char == b'_'
}

fn is_identifier_part(char: u8) -> bool {
    is_identifier_start(char) || char.is_ascii_digit()
}

fn keyword_type(lexeme: &str) -> TokenType {
    match lexeme {
        "fn" => TokenType::KwFn,
        "if" => TokenType::KwIf,
        "else" => TokenType::KwElse,
        "while" => TokenType::KwWhile,
        "const" => TokenType::KwConst,
        "return" => TokenType::KwReturn,
        "true" => TokenType::KwTrue,
        "false" => TokenType::KwFalse,
        "null" => TokenType::KwNull,
        _ => TokenType::Identifier,
    }
}

#[cfg(test)]
mod tests {
    use super::{DiagnosticKind, Lexer, TokenType};

    #[test]
    fn tokenizes_keywords_operators_numbers_and_trivia() {
        let mut lexer = Lexer::new("  const value := 42.5 // note\nreturn value");
        let result = lexer.tokenize();

        assert_eq!(7, result.tokens.len());
        assert_eq!(TokenType::KwConst, result.tokens[0].token_type);
        assert_eq!("  ", result.tokens[0].leading_trivia);
        assert_eq!("const", result.tokens[0].lexeme);

        assert_eq!(TokenType::Identifier, result.tokens[1].token_type);
        assert_eq!("value", result.tokens[1].lexeme);

        assert_eq!(TokenType::ShortDecl, result.tokens[2].token_type);
        assert_eq!(":=", result.tokens[2].lexeme);

        assert_eq!(TokenType::Number, result.tokens[3].token_type);
        assert_eq!("42.5", result.tokens[3].lexeme);

        assert_eq!(TokenType::KwReturn, result.tokens[4].token_type);
        assert_eq!(" // note\n", result.tokens[4].leading_trivia);

        assert_eq!(TokenType::Identifier, result.tokens[5].token_type);
        assert_eq!("value", result.tokens[5].lexeme);

        assert_eq!(TokenType::End, result.tokens[6].token_type);
        assert_eq!(0, result.diagnostics.len());
    }

    #[test]
    fn reports_unexpected_bang_token() {
        let mut lexer = Lexer::new("!");
        let result = lexer.tokenize();

        assert_eq!(1, result.diagnostics.len());
        assert_eq!(
            DiagnosticKind::UnexpectedTokenBang,
            result.diagnostics[0].kind
        );
        assert_eq!("Unexpected token !", result.diagnostics[0].message());
        assert_eq!(1, result.tokens.len());
        assert_eq!(TokenType::End, result.tokens[0].token_type);
    }

    #[test]
    fn reports_unterminated_string_and_keeps_parsed_text() {
        let mut lexer = Lexer::new("\"hello");
        let result = lexer.tokenize();

        assert_eq!(2, result.tokens.len());
        assert_eq!(TokenType::String, result.tokens[0].token_type);
        assert_eq!("hello", result.tokens[0].lexeme);

        assert_eq!(1, result.diagnostics.len());
        assert_eq!(
            DiagnosticKind::UnterminatedString,
            result.diagnostics[0].kind
        );
        assert_eq!("Unterminated string", result.diagnostics[0].message());
    }

    #[test]
    fn parses_comparison_and_arrow_operators() {
        let mut lexer = Lexer::new("a==b != c <= d >= e => f");
        let result = lexer.tokenize();

        let expected = [
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

        assert_eq!(expected.len(), result.tokens.len());
        for (index, token_type) in expected.iter().enumerate() {
            assert_eq!(*token_type, result.tokens[index].token_type);
        }
        assert_eq!(0, result.diagnostics.len());
    }
}
