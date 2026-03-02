#include "lune/codegen.hpp"
#include "lune/gc.hpp"
#include "lune/lexer.hpp"
#include "lune/parser.hpp"

#include <cassert>
#include <filesystem>
#include <iostream>

namespace {

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

} // namespace

int main() {
    test_lexer();
    test_jit();
    test_gc();
    test_aot();
    std::cout << "All tests passed\n";
}
