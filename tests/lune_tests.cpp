#include "lune/codegen.hpp"
#include "lune/gc.hpp"
#include "lune/lexer.hpp"
#include "lune/parser.hpp"
#include "lune/pretty_printer.hpp"

#include <cassert>
#include <chrono>
#include <filesystem>
#include <iostream>
#include <sstream>

namespace {

using Clock = std::chrono::steady_clock;

std::string build_large_program(std::size_t function_count, std::size_t statements_per_function) {
    std::ostringstream out;
    out << "const seed = 1\n";
    for (std::size_t i = 0; i < function_count; ++i) {
        out << "fn f" << i << "() {\n";
        out << "  x := " << (i % 7) << "\n";
        for (std::size_t j = 0; j < statements_per_function; ++j) {
            out << "  x = x + " << ((j % 9) + 1) << "\n";
        }
        out << "  return x\n";
        out << "}\n";
    }
    out << "fn main() { return f0() }\n";
    return out.str();
}

void test_lexer() {
    lune::Lexer lexer("const x = 42 fn main() { return x }");
    const auto tokens = lexer.tokenize();
    assert(!tokens.empty());
    assert(tokens[0].type == lune::TokenType::KwConst);
    assert(tokens[1].type == lune::TokenType::Identifier);
}

void test_lexer_while_keyword() {
    lune::Lexer lexer("while true { return 1 }");
    const auto tokens = lexer.tokenize();
    assert(tokens.size() >= 2);
    assert(tokens[0].type == lune::TokenType::KwWhile);
}


void test_lexer_trivia_and_diagnostics() {
    lune::Lexer lexer(R"(  // lead
const x = !
")");
    const auto tokens = lexer.tokenize();
    assert(tokens.size() >= 4);
    assert(tokens[0].type == lune::TokenType::KwConst);
    assert(tokens[0].leading_trivia.find("// lead") != std::string::npos);

    const auto& diagnostics = lexer.diagnostics();
    assert(diagnostics.size() >= 2);
    assert(diagnostics[0].message.find("Unexpected token !") != std::string::npos);
}


void test_parser_while_statement() {
    lune::Lexer lexer(R"(
        fn main() {
            i := 0
            while i < 3 {
                i = i + 1
            }
            return i
        }
    )");
    lune::Parser parser(lexer.tokenize());
    const auto program = parser.parse_program();
    assert(parser.diagnostics().empty());
    assert(program.items.size() == 1);

    const auto* fn = std::get_if<lune::FunctionDecl>(&program.items[0]->node);
    assert(fn != nullptr);
    assert(fn->body.statements.size() == 3);
    assert(std::holds_alternative<lune::WhileStmt>(fn->body.statements[1]->node));
}


void test_parser_error_recovery_progress() {
    lune::Lexer lexer(R"(
        fn main() {
            while {
                return 1
            }
            return 2
        }
    )");
    lune::Parser parser(lexer.tokenize());
    const auto program = parser.parse_program();

    assert(!program.items.empty());
    assert(!parser.diagnostics().empty());
}

void test_parser_diagnostics() {
    lune::Lexer lexer("fn main( { return 1 }");
    lune::Parser parser(lexer.tokenize());
    auto program = parser.parse_program();
    (void)program;
    const auto& diagnostics = parser.diagnostics();
    assert(!diagnostics.empty());
    assert(diagnostics[0].message.find("Expected") != std::string::npos);
}

void test_parser_if_else_statement() {
    lune::Lexer lexer(R"(
        fn main() {
            if true {
                return 1
            } else {
                return 2
            }
        }
    )");
    lune::Parser parser(lexer.tokenize());
    const auto program = parser.parse_program();

    assert(parser.diagnostics().empty());
    assert(program.items.size() == 1);

    const auto* fn = std::get_if<lune::FunctionDecl>(&program.items[0]->node);
    assert(fn != nullptr);
    assert(fn->body.statements.size() == 1);
    assert(std::holds_alternative<lune::IfStmt>(fn->body.statements[0]->node));
}


void test_jit_while_loop() {
    lune::Lexer lexer(R"(
        fn main() {
            i := 1
            total := 0
            while i <= 5 {
                total = total + i
                i = i + 1
            }
            return total
        }
    )");
    lune::Parser parser(lexer.tokenize());
    auto program = parser.parse_program();

    lune::Codegen codegen("jit_while_test");
    codegen.compile(program);
    const auto value = codegen.run_jit_main();
    assert(value == 15.0);
}

void test_jit() {
    lune::Lexer lexer(R"(
        fn main() {
            x := 40
            x = x + 2
            return x
        }
    )");
    lune::Parser parser(lexer.tokenize());
    auto program = parser.parse_program();

    lune::Codegen codegen("jit_test");
    codegen.compile(program);
    const auto value = codegen.run_jit_main();
    assert(value == 42.0);
}

void test_jit_function_call_and_modulo() {
    lune::Lexer lexer(R"(
        fn reduce(x, y) {
            return (x % y) + 1
        }

        fn main() {
            return reduce(20, 6)
        }
    )");
    lune::Parser parser(lexer.tokenize());
    auto program = parser.parse_program();

    lune::Codegen codegen("jit_function_call_modulo_test");
    codegen.compile(program);
    const auto value = codegen.run_jit_main();
    assert(value == 3.0);
}

void test_jit_if_else_branching() {
    lune::Lexer lexer(R"(
        fn main() {
            x := 2
            if x > 5 {
                return 100
            } else {
                return 200
            }
        }
    )");
    lune::Parser parser(lexer.tokenize());
    auto program = parser.parse_program();

    lune::Codegen codegen("jit_if_else_branching_test");
    codegen.compile(program);
    const auto value = codegen.run_jit_main();
    assert(value == 200.0);
}

struct Node : lune::GCObject {
    explicit Node(int id) : id(id) {}
    int id;
};

void test_gc() {
    lune::GarbageCollector gc;
    auto* root = gc.allocate<Node>(1);
    auto* child = gc.allocate<Node>(2);
    root->edges.push_back(child);

    gc.mark(root);
    gc.sweep();
    assert(gc.live_objects() == 2);

    gc.sweep();
    assert(gc.live_objects() == 0);
}

void test_aot() {
    lune::Lexer lexer("fn main() { return 7 }");
    lune::Parser parser(lexer.tokenize());
    auto program = parser.parse_program();

    lune::Codegen codegen("aot_test");
    codegen.compile(program);

    const auto path = std::filesystem::temp_directory_path() / "lune_test.o";
    codegen.emit_object_file(path.string());
    assert(std::filesystem::exists(path));
    std::filesystem::remove(path);
}


void test_pretty_printer() {
    lune::Lexer lexer(R"(
        const seed = 1
        fn main() {
            x := seed + 1
            if x > 1 {
                while x < 4 {
                    x = x + 1
                }
                return x
            } else {
                return 0
            }
        }
    )");
    lune::Parser parser(lexer.tokenize());
    const auto program = parser.parse_program();

    const auto rendered = lune::pretty_print(program);
    const std::string expected =
        "const seed = 1\n"
        "fn main() {\n"
        "  x := (seed + 1)\n"
        "  if (x > 1) {\n"
        "    while (x < 4) {\n"
        "      x = (x + 1)\n"
        "    }\n"
        "    return x\n"
        "  } else {\n"
        "    return 0\n"
        "  }\n"
        "}";
    assert(rendered == expected);
}


void test_performance_while_jit() {
    lune::Lexer lexer(R"(
        fn main() {
            i := 1
            acc := 0
            while i <= 10000 {
                acc = acc + i
                i = i + 1
            }
            return acc
        }
    )");

    const auto parse_start = Clock::now();
    lune::Parser parser(lexer.tokenize());
    auto program = parser.parse_program();
    const auto parse_elapsed = std::chrono::duration_cast<std::chrono::milliseconds>(Clock::now() - parse_start);

    const auto jit_start = Clock::now();
    lune::Codegen codegen("jit_while_perf_test");
    codegen.compile(program);
    const auto value = codegen.run_jit_main();
    const auto jit_elapsed = std::chrono::duration_cast<std::chrono::milliseconds>(Clock::now() - jit_start);

    std::cout << "perf: while parse took " << parse_elapsed.count() << "ms"
              << ", while jit+run took " << jit_elapsed.count() << "ms\n";

    assert(value == 50005000.0);
    assert(parse_elapsed.count() < 1000);
    assert(jit_elapsed.count() < 3000);
}

void test_performance_timings() {
    constexpr std::size_t iterations = 150;
    const auto source = build_large_program(12, 40);

    auto lexer_start = Clock::now();
    std::size_t total_tokens = 0;
    for (std::size_t i = 0; i < iterations; ++i) {
        lune::Lexer lexer(source);
        total_tokens += lexer.tokenize().size();
    }
    const auto lexer_elapsed = std::chrono::duration_cast<std::chrono::milliseconds>(Clock::now() - lexer_start);

    auto parser_start = Clock::now();
    std::size_t total_items = 0;
    for (std::size_t i = 0; i < iterations; ++i) {
        lune::Lexer lexer(source);
        lune::Parser parser(lexer.tokenize());
        total_items += parser.parse_program().items.size();
    }
    const auto parser_elapsed = std::chrono::duration_cast<std::chrono::milliseconds>(Clock::now() - parser_start);

    std::cout << "perf: tokenize " << iterations << "x took " << lexer_elapsed.count() << "ms"
              << ", parse " << iterations << "x took " << parser_elapsed.count() << "ms\n";

    assert(total_tokens > 0);
    assert(total_items > 0);
    assert(lexer_elapsed.count() < 2000);
    assert(parser_elapsed.count() < 4000);
}

} // namespace

int main() {
    test_lexer();
    test_lexer_while_keyword();
    test_lexer_trivia_and_diagnostics();
    test_parser_while_statement();
    test_parser_error_recovery_progress();
    test_parser_diagnostics();
    test_parser_if_else_statement();
    test_jit_while_loop();
    test_jit();
    test_jit_function_call_and_modulo();
    test_jit_if_else_branching();
    test_gc();
    test_aot();
    test_pretty_printer();
    test_performance_while_jit();
    test_performance_timings();
    std::cout << "All tests passed\n";
}
