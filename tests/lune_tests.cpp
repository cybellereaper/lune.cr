#include "lune/codegen.hpp"
#include "lune/gc.hpp"
#include "lune/lexer.hpp"
#include "lune/parser.hpp"

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
    test_jit();
    test_gc();
    test_aot();
    test_performance_timings();
    std::cout << "All tests passed\n";
}
