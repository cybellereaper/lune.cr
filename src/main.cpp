#include "lune/codegen.hpp"
#include "lune/lexer.hpp"
#include "lune/parser.hpp"

#include <fstream>
#include <iostream>
#include <sstream>
#include <stdexcept>

int main(int argc, char** argv) {
    try {
        if (argc < 2) {
            std::cerr << "Usage: lune <file.lune> [--jit|--aot <out.o>]\n";
            return 1;
        }

        std::ifstream in(argv[1]);
        if (!in) {
            throw std::runtime_error("Unable to open input file");
        }
        std::stringstream buffer;
        buffer << in.rdbuf();

        lune::Lexer lexer(buffer.str());
        auto tokens = lexer.tokenize();
        lune::Parser parser(std::move(tokens));
        auto program = parser.parse_program();

        lune::Codegen codegen("lune_module");
        codegen.compile(program);

        if (argc >= 3 && std::string(argv[2]) == "--aot") {
            const std::string out = argc >= 4 ? argv[3] : "a.out.o";
            codegen.emit_object_file(out);
            std::cout << "Wrote object file: " << out << '\n';
            return 0;
        }

        const auto result = codegen.run_jit_main();
        std::cout << result << '\n';
        return 0;
    } catch (const std::exception& ex) {
        std::cerr << "error: " << ex.what() << '\n';
        return 1;
    }
}
