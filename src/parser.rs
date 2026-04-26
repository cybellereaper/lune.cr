use crate::ast::{AstNode, Program};
use crate::{Token, TokenType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParserDiagnosticKind {
    InvalidNumber,
}

impl ParserDiagnosticKind {
    pub fn message(self) -> &'static str {
        match self {
            ParserDiagnosticKind::InvalidNumber => "Invalid numeric literal",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParserDiagnostic {
    pub kind: ParserDiagnosticKind,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseResult {
    pub program: Program,
    pub diagnostics: Vec<ParserDiagnostic>,
}

pub struct Parser<'a> {
    tokens: &'a [Token],
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self { tokens }
    }

    pub fn parse(&self) -> ParseResult {
        let mut nodes = Vec::new();
        let mut diagnostics = Vec::new();

        for token in self.tokens.iter().filter(|token| !is_end_token(token)) {
            match token_to_node(token) {
                Ok(Some(node)) => nodes.push(node),
                Ok(None) => {}
                Err(kind) => diagnostics.push(ParserDiagnostic {
                    kind,
                    line: token.line,
                    column: token.column,
                }),
            }
        }

        ParseResult {
            program: Program::new(nodes),
            diagnostics,
        }
    }
}

fn is_end_token(token: &Token) -> bool {
    token.token_type == TokenType::End
}

fn token_to_node(token: &Token) -> Result<Option<AstNode>, ParserDiagnosticKind> {
    let node = match token.token_type {
        TokenType::Number => Some(AstNode::Number(
            token
                .lexeme
                .parse::<f64>()
                .map_err(|_| ParserDiagnosticKind::InvalidNumber)?,
        )),
        TokenType::String => Some(AstNode::String(token.lexeme.clone())),
        TokenType::KwTrue => Some(AstNode::Bool(true)),
        TokenType::KwFalse => Some(AstNode::Bool(false)),
        TokenType::KwNull => Some(AstNode::Null),
        TokenType::Identifier => Some(AstNode::Identifier(token.lexeme.clone())),
        _ => None,
    };

    Ok(node)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Lexer;

    #[test]
    fn parses_basic_literals_and_identifiers_into_ast_nodes() {
        let mut lexer = Lexer::new("42 \"ok\" true false null foo");
        let lexed = lexer.tokenize();
        let parser = Parser::new(&lexed.tokens);

        let result = parser.parse();

        assert!(result.diagnostics.is_empty());
        assert_eq!(
            result.program.nodes,
            vec![
                AstNode::Number(42.0),
                AstNode::String("ok".to_owned()),
                AstNode::Bool(true),
                AstNode::Bool(false),
                AstNode::Null,
                AstNode::Identifier("foo".to_owned())
            ]
        );
    }

    #[test]
    fn reports_invalid_number_diagnostic_when_number_token_is_malformed() {
        let tokens = [Token {
            token_type: TokenType::Number,
            lexeme: "1.2.3".to_owned(),
            leading_trivia: String::new(),
            line: 10,
            column: 4,
        }];
        let parser = Parser::new(&tokens);

        let result = parser.parse();

        assert_eq!(result.program.nodes, Vec::<AstNode>::new());
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(
            result.diagnostics[0].kind,
            ParserDiagnosticKind::InvalidNumber
        );
        assert_eq!(result.diagnostics[0].line, 10);
        assert_eq!(result.diagnostics[0].column, 4);
    }
}
