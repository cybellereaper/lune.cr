module Lune
  enum TokenType
    End
    Identifier
    Number
    String
    KwFn
    KwIf
    KwElse
    KwWhile
    KwConst
    KwReturn
    KwTrue
    KwFalse
    KwNull
    LParen
    RParen
    LBrace
    RBrace
    Comma
    Dot
    Colon
    Plus
    Minus
    Star
    Slash
    Percent
    Assign
    ShortDecl
    Arrow
    Eq
    Ne
    Lt
    Le
    Gt
    Ge
  end

  TOKEN_TYPE_NAMES = {
    TokenType::End        => "end",
    TokenType::Identifier => "identifier",
    TokenType::Number     => "number",
    TokenType::String     => "string",
    TokenType::KwFn       => "fn",
    TokenType::KwIf       => "if",
    TokenType::KwElse     => "else",
    TokenType::KwWhile    => "while",
    TokenType::KwConst    => "const",
    TokenType::KwReturn   => "return",
    TokenType::KwTrue     => "true",
    TokenType::KwFalse    => "false",
    TokenType::KwNull     => "null",
    TokenType::LParen     => "(",
    TokenType::RParen     => ")",
    TokenType::LBrace     => "{",
    TokenType::RBrace     => "}",
    TokenType::Comma      => ",",
    TokenType::Dot        => ".",
    TokenType::Colon      => ":",
    TokenType::Plus       => "+",
    TokenType::Minus      => "-",
    TokenType::Star       => "*",
    TokenType::Slash      => "/",
    TokenType::Percent    => "%",
    TokenType::Assign     => "=",
    TokenType::ShortDecl  => ":=",
    TokenType::Arrow      => "=>",
    TokenType::Eq         => "==",
    TokenType::Ne         => "!=",
    TokenType::Lt         => "<",
    TokenType::Le         => "<=",
    TokenType::Gt         => ">",
    TokenType::Ge         => ">=",
  }

  struct Token
    getter token_type : TokenType
    getter lexeme : String
    getter leading_trivia : String
    getter line : Int32
    getter column : Int32

    def initialize(@token_type : TokenType, @lexeme : String, @leading_trivia : String, @line : Int32, @column : Int32)
    end
  end

  enum DiagnosticKind
    UnexpectedTokenBang
    UnexpectedCharacter
    UnterminatedString

    def message : String
      case self
      when DiagnosticKind::UnexpectedTokenBang
        "Unexpected token !"
      when DiagnosticKind::UnexpectedCharacter
        "Unexpected character in input"
      when DiagnosticKind::UnterminatedString
        "Unterminated string"
      else
        raise "unreachable diagnostic kind: #{self}"
      end
    end
  end

  struct Diagnostic
    getter kind : DiagnosticKind
    getter line : Int32
    getter column : Int32

    def initialize(@kind : DiagnosticKind, @line : Int32, @column : Int32)
    end

    def message : String
      kind.message
    end
  end

  struct LexerResult
    getter tokens : Array(Token)
    getter diagnostics : Array(Diagnostic)

    def initialize(@tokens : Array(Token), @diagnostics : Array(Diagnostic))
    end
  end

  private struct TokenStart
    getter offset : Int32
    getter line : Int32
    getter column : Int32

    def initialize(@offset : Int32, @line : Int32, @column : Int32)
    end
  end

  private class Scanner
    getter source : String
    getter offset : Int32 = 0
    getter line : Int32 = 1
    getter column : Int32 = 1

    def initialize(@source : String)
    end

    def at_end? : Bool
      offset >= source.bytesize
    end

    def peek : Char?
      return nil if at_end?
      source.byte_at(offset).unsafe_chr
    end

    def peek_next : Char?
      index = offset + 1
      return nil if index >= source.bytesize
      source.byte_at(index).unsafe_chr
    end

    def advance : Char?
      return nil if at_end?

      char = source.byte_at(offset).unsafe_chr
      @offset += 1
      if char == '\n'
        @line += 1
        @column = 1
      else
        @column += 1
      end
      char
    end

    def match(expected : Char) : Bool
      return false unless peek == expected
      advance
      true
    end
  end

  class Lexer
    KEYWORDS = {
      "fn"     => TokenType::KwFn,
      "if"     => TokenType::KwIf,
      "else"   => TokenType::KwElse,
      "while"  => TokenType::KwWhile,
      "const"  => TokenType::KwConst,
      "return" => TokenType::KwReturn,
      "true"   => TokenType::KwTrue,
      "false"  => TokenType::KwFalse,
      "null"   => TokenType::KwNull,
    }

    SINGLE_CHAR_TOKENS = {
      '(' => TokenType::LParen,
      ')' => TokenType::RParen,
      '{' => TokenType::LBrace,
      '}' => TokenType::RBrace,
      ',' => TokenType::Comma,
      '.' => TokenType::Dot,
      '+' => TokenType::Plus,
      '-' => TokenType::Minus,
      '*' => TokenType::Star,
      '/' => TokenType::Slash,
      '%' => TokenType::Percent,
    }

    @scanner : Scanner

    def initialize(source : String)
      @scanner = Scanner.new(source)
    end

    def tokenize : LexerResult
      tokens = [] of Token
      diagnostics = [] of Diagnostic

      until @scanner.at_end?
        trivia_start = @scanner.offset
        skip_trivia
        trivia = slice(trivia_start, @scanner.offset)
        break if @scanner.at_end?

        start = TokenStart.new(@scanner.offset, @scanner.line, @scanner.column)
        current = @scanner.advance
        next unless current

        scan_token(tokens, diagnostics, start, trivia, current)
      end

      tokens << Token.new(TokenType::End, "", "", @scanner.line, @scanner.column)
      LexerResult.new(tokens, diagnostics)
    end

    private def scan_token(tokens : Array(Token), diagnostics : Array(Diagnostic), start : TokenStart, trivia : String, current : Char) : Nil
      if token_type = SINGLE_CHAR_TOKENS[current]?
        append_token(tokens, token_type, trivia, start, @scanner.offset)
        return
      end

      case current
      when ':'
        if @scanner.match('=')
          append_token(tokens, TokenType::ShortDecl, trivia, start, @scanner.offset)
        else
          append_token(tokens, TokenType::Colon, trivia, start, @scanner.offset)
        end
      when '='
        if @scanner.match('=')
          append_token(tokens, TokenType::Eq, trivia, start, @scanner.offset)
        elsif @scanner.match('>')
          append_token(tokens, TokenType::Arrow, trivia, start, @scanner.offset)
        else
          append_token(tokens, TokenType::Assign, trivia, start, @scanner.offset)
        end
      when '!'
        if @scanner.match('=')
          append_token(tokens, TokenType::Ne, trivia, start, @scanner.offset)
        else
          append_diagnostic(diagnostics, DiagnosticKind::UnexpectedTokenBang, start)
        end
      when '<'
        if @scanner.match('=')
          append_token(tokens, TokenType::Le, trivia, start, @scanner.offset)
        else
          append_token(tokens, TokenType::Lt, trivia, start, @scanner.offset)
        end
      when '>'
        if @scanner.match('=')
          append_token(tokens, TokenType::Ge, trivia, start, @scanner.offset)
        else
          append_token(tokens, TokenType::Gt, trivia, start, @scanner.offset)
        end
      when '"'
        scan_string(tokens, diagnostics, start, trivia)
      else
        if ascii_digit?(current)
          scan_number(tokens, start, trivia)
        elsif identifier_start?(current)
          scan_identifier(tokens, start, trivia)
        else
          append_diagnostic(diagnostics, DiagnosticKind::UnexpectedCharacter, start)
        end
      end
    end

    private def scan_identifier(tokens : Array(Token), start : TokenStart, trivia : String) : Nil
      while (next_char = @scanner.peek) && identifier_part?(next_char)
        @scanner.advance
      end

      lexeme = slice(start.offset, @scanner.offset)
      token_type = KEYWORDS[lexeme]? || TokenType::Identifier
      append_token(tokens, token_type, trivia, start, @scanner.offset)
    end

    private def scan_number(tokens : Array(Token), start : TokenStart, trivia : String) : Nil
      while (next_char = @scanner.peek) && ascii_digit?(next_char)
        @scanner.advance
      end

      if @scanner.peek == '.'
        fraction_start = @scanner.peek_next
        if fraction_start && ascii_digit?(fraction_start)
          @scanner.advance
          while (digit = @scanner.peek) && ascii_digit?(digit)
            @scanner.advance
          end
        end
      end

      append_token(tokens, TokenType::Number, trivia, start, @scanner.offset)
    end

    private def scan_string(tokens : Array(Token), diagnostics : Array(Diagnostic), start : TokenStart, trivia : String) : Nil
      string_content_start = @scanner.offset

      while (next_char = @scanner.peek)
        break if next_char == '"'
        @scanner.advance
      end

      if @scanner.at_end?
        append_token_slice(tokens, TokenType::String, trivia, start, string_content_start, @scanner.offset)
        append_diagnostic(diagnostics, DiagnosticKind::UnterminatedString, start)
        return
      end

      string_content_end = @scanner.offset
      @scanner.advance
      append_token_slice(tokens, TokenType::String, trivia, start, string_content_start, string_content_end)
    end

    private def skip_trivia : Nil
      until @scanner.at_end?
        current = @scanner.peek
        break unless current

        if ascii_whitespace?(current)
          @scanner.advance
          next
        end

        if current == '/' && @scanner.peek_next == '/'
          @scanner.advance
          @scanner.advance
          while (next_char = @scanner.peek) && next_char != '\n'
            @scanner.advance
          end
          next
        end

        break
      end
    end

    private def append_token(tokens : Array(Token), token_type : TokenType, trivia : String, start : TokenStart, end_offset : Int32) : Nil
      append_token_slice(tokens, token_type, trivia, start, start.offset, end_offset)
    end

    private def append_token_slice(tokens : Array(Token), token_type : TokenType, trivia : String, start : TokenStart, start_offset : Int32, end_offset : Int32) : Nil
      tokens << Token.new(
        token_type,
        slice(start_offset, end_offset),
        trivia,
        start.line,
        start.column
      )
    end

    private def append_diagnostic(diagnostics : Array(Diagnostic), kind : DiagnosticKind, start : TokenStart) : Nil
      diagnostics << Diagnostic.new(kind, start.line, start.column)
    end

    private def slice(start_offset : Int32, end_offset : Int32) : String
      @scanner.source.byte_slice(start_offset, end_offset - start_offset)
    end

    private def ascii_digit?(char : Char) : Bool
      char >= '0' && char <= '9'
    end

    private def ascii_whitespace?(char : Char) : Bool
      char == ' ' || char == '\t' || char == '\n' || char == '\r' || char == '\v' || char == '\f'
    end

    private def ascii_alpha?(char : Char) : Bool
      (char >= 'a' && char <= 'z') || (char >= 'A' && char <= 'Z')
    end

    private def identifier_start?(char : Char) : Bool
      ascii_alpha?(char) || char == '_'
    end

    private def identifier_part?(char : Char) : Bool
      identifier_start?(char) || ascii_digit?(char)
    end
  end
end
