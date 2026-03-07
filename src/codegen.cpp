#include "lune/codegen.hpp"

#include <llvm/ExecutionEngine/Orc/LLJIT.h>
#include <llvm/IR/BasicBlock.h>
#include <llvm/IR/IRBuilder.h>
#include <llvm/IR/Verifier.h>
#include <llvm/IR/LegacyPassManager.h>
#include <llvm/MC/TargetRegistry.h>
#include <llvm/Support/FileSystem.h>
#include <llvm/TargetParser/Host.h>
#include <llvm/Support/TargetSelect.h>
#include <llvm/Target/TargetMachine.h>

#include <iostream>
#include <stdexcept>
#include <unordered_map>

namespace lune {

class Codegen::Impl {
public:
    Impl(std::string module_name)
        : context(std::make_unique<llvm::LLVMContext>()),
          module(std::make_unique<llvm::Module>(module_name, *context)),
          builder(std::make_unique<llvm::IRBuilder<>>(*context)) {
        llvm::InitializeNativeTarget();
        llvm::InitializeNativeTargetAsmPrinter();
        llvm::InitializeNativeTargetAsmParser();
    }

    llvm::Value* emit_expr(const ExprPtr& expr) {
        return std::visit([this](const auto& node) -> llvm::Value* { return emit(node); }, expr->node);
    }

    void emit_stmt(const StmtPtr& stmt) {
        std::visit([this](const auto& node) { emit(node); }, stmt->node);
    }

    llvm::Value* emit(const NumberExpr& e) { return llvm::ConstantFP::get(llvm::Type::getDoubleTy(*context), e.value); }
    llvm::Value* emit(const BoolExpr& e) { return llvm::ConstantFP::get(llvm::Type::getDoubleTy(*context), e.value ? 1.0 : 0.0); }
    llvm::Value* emit(const NullExpr&) { return llvm::ConstantFP::get(llvm::Type::getDoubleTy(*context), 0.0); }
    llvm::Value* emit(const StringExpr&) { return llvm::ConstantFP::get(llvm::Type::getDoubleTy(*context), 0.0); }

    llvm::Value* emit(const IdentifierExpr& e) {
        auto it = named_values.find(e.name);
        if (it == named_values.end()) throw std::runtime_error("Unknown identifier: " + e.name);
        return builder->CreateLoad(llvm::Type::getDoubleTy(*context), it->second, e.name);
    }

    llvm::Value* emit(const BinaryExpr& e) {
        auto* lhs = emit_expr(e.lhs);
        auto* rhs = emit_expr(e.rhs);
        if (e.op == "+") return builder->CreateFAdd(lhs, rhs, "addtmp");
        if (e.op == "-") return builder->CreateFSub(lhs, rhs, "subtmp");
        if (e.op == "*") return builder->CreateFMul(lhs, rhs, "multmp");
        if (e.op == "/") return builder->CreateFDiv(lhs, rhs, "divtmp");
        if (e.op == "%") {
            auto* div = builder->CreateFDiv(lhs, rhs, "mod_div");
            auto* floored = builder->CreateUnaryIntrinsic(llvm::Intrinsic::floor, div);
            return builder->CreateFSub(lhs, builder->CreateFMul(floored, rhs), "modtmp");
        }
        llvm::CmpInst::Predicate pred{};
        if (e.op == "==") pred = llvm::CmpInst::FCMP_OEQ;
        else if (e.op == "!=") pred = llvm::CmpInst::FCMP_ONE;
        else if (e.op == "<") pred = llvm::CmpInst::FCMP_OLT;
        else if (e.op == "<=") pred = llvm::CmpInst::FCMP_OLE;
        else if (e.op == ">") pred = llvm::CmpInst::FCMP_OGT;
        else if (e.op == ">=") pred = llvm::CmpInst::FCMP_OGE;
        else throw std::runtime_error("Unsupported binary operator: " + e.op);
        return builder->CreateUIToFP(builder->CreateFCmp(pred, lhs, rhs), llvm::Type::getDoubleTy(*context), "booltmp");
    }

    llvm::Value* emit(const CallExpr& e) {
        auto* callee = module->getFunction(e.callee);
        if (!callee) throw std::runtime_error("Unknown function: " + e.callee);
        std::vector<llvm::Value*> args;
        args.reserve(e.args.size());
        for (const auto& arg : e.args) {
            args.push_back(emit_expr(arg));
        }
        return builder->CreateCall(callee, args, "calltmp");
    }

    llvm::Value* emit(const IfExpr&) { throw std::runtime_error("If expression not lowered yet"); }

    void emit(const ExprStmt& s) { (void)emit_expr(s.expr); }

    llvm::AllocaInst* create_alloca(llvm::Function* fn, const std::string& name) {
        llvm::IRBuilder<> local_builder(&fn->getEntryBlock(), fn->getEntryBlock().begin());
        return local_builder.CreateAlloca(llvm::Type::getDoubleTy(*context), nullptr, name);
    }

    void emit(const ConstDeclStmt& s) {
        auto* fn = builder->GetInsertBlock()->getParent();
        auto* alloca = create_alloca(fn, s.name);
        builder->CreateStore(emit_expr(s.expr), alloca);
        named_values[s.name] = alloca;
    }

    void emit(const ShortDeclStmt& s) { emit(ConstDeclStmt{.name = s.name, .expr = s.expr}); }

    void emit(const AssignStmt& s) {
        auto it = named_values.find(s.name);
        if (it == named_values.end()) throw std::runtime_error("Unknown assignment target: " + s.name);
        builder->CreateStore(emit_expr(s.expr), it->second);
    }

    void emit(const ReturnStmt& s) {
        if (!s.expr) {
            builder->CreateRet(llvm::ConstantFP::get(llvm::Type::getDoubleTy(*context), 0.0));
            return;
        }
        builder->CreateRet(emit_expr(*s.expr));
    }

    void emit(const BlockStmt& s) {
        for (const auto& stmt : s.statements) {
            emit_stmt(stmt);
            if (builder->GetInsertBlock()->getTerminator()) {
                break;
            }
        }
    }

    void emit(const IfStmt& s) {
        auto* fn = builder->GetInsertBlock()->getParent();
        auto* cond = emit_expr(s.condition);
        cond = builder->CreateFCmpONE(cond, llvm::ConstantFP::get(llvm::Type::getDoubleTy(*context), 0.0), "ifcond");

        auto* then_bb = llvm::BasicBlock::Create(*context, "then", fn);
        auto* else_bb = llvm::BasicBlock::Create(*context, "else");
        auto* merge_bb = llvm::BasicBlock::Create(*context, "ifend");
        builder->CreateCondBr(cond, then_bb, else_bb);

        builder->SetInsertPoint(then_bb);
        emit(s.then_branch);
        if (!builder->GetInsertBlock()->getTerminator()) builder->CreateBr(merge_bb);

        fn->insert(fn->end(), else_bb);
        builder->SetInsertPoint(else_bb);
        if (s.else_branch) emit(*s.else_branch);
        if (!builder->GetInsertBlock()->getTerminator()) builder->CreateBr(merge_bb);

        fn->insert(fn->end(), merge_bb);
        builder->SetInsertPoint(merge_bb);
    }


    void emit(const WhileStmt& s) {
        if (builder->GetInsertBlock()->getTerminator()) {
            return;
        }

        auto* fn = builder->GetInsertBlock()->getParent();

        auto* cond_bb = llvm::BasicBlock::Create(*context, "while.cond", fn);
        auto* body_bb = llvm::BasicBlock::Create(*context, "while.body");
        auto* end_bb = llvm::BasicBlock::Create(*context, "while.end");

        builder->CreateBr(cond_bb);

        builder->SetInsertPoint(cond_bb);
        auto* cond = emit_expr(s.condition);
        cond = builder->CreateFCmpONE(cond, llvm::ConstantFP::get(llvm::Type::getDoubleTy(*context), 0.0), "whilecond");
        builder->CreateCondBr(cond, body_bb, end_bb);

        fn->insert(fn->end(), body_bb);
        builder->SetInsertPoint(body_bb);
        emit(s.body);
        if (!builder->GetInsertBlock()->getTerminator()) {
            builder->CreateBr(cond_bb);
        }

        fn->insert(fn->end(), end_bb);
        builder->SetInsertPoint(end_bb);
    }
    void emit(const FunctionDecl& s) {
        std::vector<llvm::Type*> args(s.params.size(), llvm::Type::getDoubleTy(*context));
        auto* type = llvm::FunctionType::get(llvm::Type::getDoubleTy(*context), args, false);
        auto* fn = llvm::Function::Create(type, llvm::Function::ExternalLinkage, s.name, module.get());

        std::size_t idx = 0;
        for (auto& arg : fn->args()) {
            arg.setName(s.params[idx++]);
        }

        auto* bb = llvm::BasicBlock::Create(*context, "entry", fn);
        builder->SetInsertPoint(bb);
        named_values.clear();

        for (auto& arg : fn->args()) {
            auto* alloca = create_alloca(fn, std::string(arg.getName()));
            builder->CreateStore(&arg, alloca);
            named_values[std::string(arg.getName())] = alloca;
        }

        emit(s.body);
        if (!builder->GetInsertBlock()->getTerminator()) {
            builder->CreateRet(llvm::ConstantFP::get(llvm::Type::getDoubleTy(*context), 0.0));
        }

        if (llvm::verifyFunction(*fn, &llvm::errs())) {
            fn->eraseFromParent();
            throw std::runtime_error("Function verification failed: " + s.name);
        }
    }

    llvm::orc::ThreadSafeModule to_tsm() {
        return llvm::orc::ThreadSafeModule(std::move(module), std::move(context));
    }

    std::unique_ptr<llvm::LLVMContext> context;
    std::unique_ptr<llvm::Module> module;
    std::unique_ptr<llvm::IRBuilder<>> builder;
    std::unordered_map<std::string, llvm::AllocaInst*> named_values;
};

Codegen::Codegen(std::string module_name) : impl_(std::make_unique<Impl>(std::move(module_name))) {}

Codegen::~Codegen() = default;

llvm::Module& Codegen::compile(const Program& program) {
    for (const auto& item : program.items) {
        impl_->emit_stmt(item);
    }
    if (llvm::verifyModule(*impl_->module, &llvm::errs())) {
        throw std::runtime_error("Module verification failed");
    }
    return *impl_->module;
}

double Codegen::run_jit_main() {
    auto jit_or_err = llvm::orc::LLJITBuilder().create();
    if (!jit_or_err) {
        throw std::runtime_error("Could not create JIT");
    }
    auto jit = std::move(*jit_or_err);

    if (auto err = jit->addIRModule(impl_->to_tsm())) {
        throw std::runtime_error("Failed to add module to JIT");
    }

    auto sym = jit->lookup("main");
    if (!sym) {
        throw std::runtime_error("main function not found for JIT");
    }

    auto* fn = sym->toPtr<double (*)()>();
    return fn();
}

void Codegen::emit_object_file(const std::string& output_path) {
    auto triple = llvm::sys::getDefaultTargetTriple();
    impl_->module->setTargetTriple(triple);

    std::string error;
    const auto* target = llvm::TargetRegistry::lookupTarget(triple, error);
    if (!target) throw std::runtime_error(error);

    llvm::TargetOptions options;
    auto rm = std::optional<llvm::Reloc::Model>();
    auto machine = std::unique_ptr<llvm::TargetMachine>(target->createTargetMachine(triple, "generic", "", options, rm));
    impl_->module->setDataLayout(machine->createDataLayout());

    std::error_code ec;
    llvm::raw_fd_ostream out(output_path, ec, llvm::sys::fs::OF_None);
    if (ec) throw std::runtime_error("Failed to open object file output");

    llvm::legacy::PassManager pass;
    if (machine->addPassesToEmitFile(pass, out, nullptr, llvm::CodeGenFileType::ObjectFile)) {
        throw std::runtime_error("TargetMachine cannot emit object file");
    }

    pass.run(*impl_->module);
    out.flush();
}

} // namespace lune
