#pragma once

#include <memory>
#include <optional>
#include <string>
#include <utility>
#include <variant>
#include <vector>

namespace lune {

struct Expr;
struct Stmt;

using ExprPtr = std::shared_ptr<Expr>;
using StmtPtr = std::shared_ptr<Stmt>;

struct NumberExpr { double value{}; };
struct BoolExpr { bool value{}; };
struct NullExpr {};
struct StringExpr { std::string value; };
struct IdentifierExpr { std::string name; };

struct BinaryExpr {
    std::string op;
    ExprPtr lhs;
    ExprPtr rhs;
};

struct CallExpr {
    std::string callee;
    std::vector<ExprPtr> args;
};

struct IfExpr {
    ExprPtr condition;
    std::vector<StmtPtr> then_branch;
    std::vector<StmtPtr> else_branch;
};

struct Expr {
    using Variant = std::variant<NumberExpr, BoolExpr, NullExpr, StringExpr, IdentifierExpr, BinaryExpr, CallExpr, IfExpr>;
    Variant node;
};

struct ExprStmt { ExprPtr expr; };
struct ConstDeclStmt { std::string name; ExprPtr expr; };
struct ShortDeclStmt { std::string name; ExprPtr expr; };
struct AssignStmt { std::string name; ExprPtr expr; };
struct ReturnStmt { std::optional<ExprPtr> expr; };

struct BlockStmt { std::vector<StmtPtr> statements; };

struct IfStmt {
    ExprPtr condition;
    BlockStmt then_branch;
    std::optional<BlockStmt> else_branch;
};

struct FunctionDecl {
    std::string name;
    std::vector<std::string> params;
    BlockStmt body;
};

struct Stmt {
    using Variant = std::variant<ExprStmt, ConstDeclStmt, ShortDeclStmt, AssignStmt, ReturnStmt, IfStmt, FunctionDecl, BlockStmt>;
    Variant node;
};

struct Program {
    std::vector<StmtPtr> items;
};

} // namespace lune
