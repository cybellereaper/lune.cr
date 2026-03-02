#pragma once

#include "lune/ast.hpp"

#include <llvm/IR/LLVMContext.h>
#include <llvm/IR/Module.h>

#include <memory>
#include <string>

namespace lune {

class Codegen {
public:
    explicit Codegen(std::string module_name = "lune");
    ~Codegen();
    llvm::Module& compile(const Program& program);
    double run_jit_main();
    void emit_object_file(const std::string& output_path);

private:
    class Impl;
    std::unique_ptr<Impl> impl_;
};

} // namespace lune
