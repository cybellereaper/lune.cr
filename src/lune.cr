require "./lune/lexer"

module Lune
  class Cli
    def run(argv : Array(String)) : Int32
      if argv.size != 1
        STDERR.puts "Usage: lune <file.lune>"
        return 1
      end

      source_path = argv.first
      source = File.read(source_path)

      result = Lexer.new(source).tokenize
      print_tokens(result.tokens)

      return 0 if result.diagnostics.empty?

      print_diagnostics(result.diagnostics)
      1
    rescue ex : File::Error
      STDERR.puts "error: #{ex.message}"
      1
    end

    private def print_tokens(tokens : Array(Token)) : Nil
      tokens.each do |token|
        puts "#{TOKEN_TYPE_NAMES[token.token_type]}\t\"#{token.lexeme}\"\t(#{token.line}:#{token.column})"
      end
    end

    private def print_diagnostics(diagnostics : Array(Diagnostic)) : Nil
      diagnostics.each do |diagnostic|
        STDERR.puts "error: #{diagnostic.message} at #{diagnostic.line}:#{diagnostic.column}"
      end
    end
  end
end

exit Lune::Cli.new.run(ARGV)
