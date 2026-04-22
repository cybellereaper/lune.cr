require "./spec_helper"

describe Lune::Lexer do
  it "tokenizes keywords operators numbers and trivia" do
    result = Lune::Lexer.new("  const value := 42.5 // note\nreturn value").tokenize

    result.tokens.size.should eq(7)

    result.tokens[0].token_type.should eq(Lune::TokenType::KwConst)
    result.tokens[0].leading_trivia.should eq("  ")
    result.tokens[0].lexeme.should eq("const")

    result.tokens[1].token_type.should eq(Lune::TokenType::Identifier)
    result.tokens[1].lexeme.should eq("value")

    result.tokens[2].token_type.should eq(Lune::TokenType::ShortDecl)
    result.tokens[2].lexeme.should eq(":=")

    result.tokens[3].token_type.should eq(Lune::TokenType::Number)
    result.tokens[3].lexeme.should eq("42.5")

    result.tokens[4].token_type.should eq(Lune::TokenType::KwReturn)
    result.tokens[4].leading_trivia.should eq(" // note\n")

    result.tokens[5].token_type.should eq(Lune::TokenType::Identifier)
    result.tokens[5].lexeme.should eq("value")

    result.tokens[6].token_type.should eq(Lune::TokenType::End)
    result.diagnostics.should be_empty
  end

  it "reports unexpected bang token" do
    result = Lune::Lexer.new("!").tokenize

    result.diagnostics.size.should eq(1)
    result.diagnostics[0].kind.should eq(Lune::DiagnosticKind::UnexpectedTokenBang)
    result.diagnostics[0].message.should eq("Unexpected token !")
    result.tokens.size.should eq(1)
    result.tokens[0].token_type.should eq(Lune::TokenType::End)
  end

  it "reports unterminated string and keeps parsed text" do
    result = Lune::Lexer.new("\"hello").tokenize

    result.tokens.size.should eq(2)
    result.tokens[0].token_type.should eq(Lune::TokenType::String)
    result.tokens[0].lexeme.should eq("hello")

    result.diagnostics.size.should eq(1)
    result.diagnostics[0].kind.should eq(Lune::DiagnosticKind::UnterminatedString)
    result.diagnostics[0].message.should eq("Unterminated string")
  end

  it "parses terminated string without diagnostics" do
    result = Lune::Lexer.new("\"hello\"").tokenize

    result.tokens.size.should eq(2)
    result.tokens[0].token_type.should eq(Lune::TokenType::String)
    result.tokens[0].lexeme.should eq("hello")
    result.diagnostics.should be_empty
  end

  it "parses comparison and arrow operators" do
    result = Lune::Lexer.new("a==b != c <= d >= e => f").tokenize

    expected = [
      Lune::TokenType::Identifier,
      Lune::TokenType::Eq,
      Lune::TokenType::Identifier,
      Lune::TokenType::Ne,
      Lune::TokenType::Identifier,
      Lune::TokenType::Le,
      Lune::TokenType::Identifier,
      Lune::TokenType::Ge,
      Lune::TokenType::Identifier,
      Lune::TokenType::Arrow,
      Lune::TokenType::Identifier,
      Lune::TokenType::End,
    ]

    result.tokens.map(&.token_type).should eq(expected)
    result.diagnostics.should be_empty
  end

  it "captures unknown characters as diagnostics and still emits end token" do
    result = Lune::Lexer.new("@").tokenize

    result.tokens.size.should eq(1)
    result.tokens[0].token_type.should eq(Lune::TokenType::End)
    result.diagnostics.size.should eq(1)
    result.diagnostics[0].kind.should eq(Lune::DiagnosticKind::UnexpectedCharacter)
  end
end
