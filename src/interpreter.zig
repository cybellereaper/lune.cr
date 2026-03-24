const std = @import("std");
const ast = @import("ast.zig");

pub const RuntimeValue = union(enum) {
    number: f64,
    boolean: bool,

    fn asNumber(self: RuntimeValue) !f64 {
        return switch (self) {
            .number => |n| n,
            else => error.ExpectedNumber,
        };
    }

    fn truthy(self: RuntimeValue) bool {
        return switch (self) {
            .boolean => |b| b,
            .number => |n| n != 0,
        };
    }
};

const Scope = struct {
    parent: ?*Scope,
    values: std.StringHashMap(RuntimeValue),

    fn init(allocator: std.mem.Allocator, parent: ?*Scope) Scope {
        return .{ .parent = parent, .values = std.StringHashMap(RuntimeValue).init(allocator) };
    }

    fn define(self: *Scope, name: []const u8, value: RuntimeValue) !void {
        try self.values.put(name, value);
    }

    fn assign(self: *Scope, name: []const u8, value: RuntimeValue) !void {
        if (self.values.contains(name)) {
            try self.values.put(name, value);
            return;
        }
        if (self.parent) |parent| return parent.assign(name, value);
        return error.UnknownVariable;
    }

    fn get(self: *Scope, name: []const u8) !RuntimeValue {
        if (self.values.get(name)) |value| return value;
        if (self.parent) |parent| return parent.get(name);
        return error.UnknownVariable;
    }
};

pub const Interpreter = struct {
    allocator: std.mem.Allocator,
    functions: std.StringHashMap(ast.FunctionDecl),
    globals: Scope,

    pub fn init(allocator: std.mem.Allocator) Interpreter {
        return .{
            .allocator = allocator,
            .functions = std.StringHashMap(ast.FunctionDecl).init(allocator),
            .globals = Scope.init(allocator, null),
        };
    }

    pub fn deinit(self: *Interpreter) void {
        self.functions.deinit();
        self.globals.values.deinit();
    }

    pub fn runMain(self: *Interpreter, program: ast.Program) anyerror!f64 {
        try self.loadProgram(program);
        const result = try self.callFunction("main", &[_]RuntimeValue{});
        return result.asNumber();
    }

    fn loadProgram(self: *Interpreter, program: ast.Program) anyerror!void {
        for (program.items) |item| switch (item) {
            .const_decl => |decl| try self.globals.define(decl.name, try self.evalExpr(&self.globals, decl.value)),
            .function_decl => |decl| try self.functions.put(decl.name, decl),
        };
    }

    fn callFunction(self: *Interpreter, name: []const u8, args: []const RuntimeValue) anyerror!RuntimeValue {
        const function_decl = self.functions.get(name) orelse return error.UnknownFunction;
        if (function_decl.params.len != args.len) return error.InvalidArity;

        var local_scope = Scope.init(self.allocator, &self.globals);
        defer local_scope.values.deinit();

        for (function_decl.params, args) |param, arg| try local_scope.define(param, arg);

        const signal = try self.execBlock(&local_scope, function_decl.body);
        return switch (signal) {
            .returned => |value| value,
            .none => RuntimeValue{ .number = 0 },
        };
    }

    const Signal = union(enum) {
        none,
        returned: RuntimeValue,
    };

    fn execBlock(self: *Interpreter, scope: *Scope, statements: []const ast.Stmt) anyerror!Signal {
        for (statements) |statement| {
            const signal = try self.execStmt(scope, statement);
            if (signal == .returned) return signal;
        }
        return .none;
    }

    fn execStmt(self: *Interpreter, scope: *Scope, statement: ast.Stmt) anyerror!Signal {
        switch (statement) {
            .var_decl => |decl| {
                try scope.define(decl.name, try self.evalExpr(scope, decl.value));
                return .none;
            },
            .assign => |assignment| {
                try scope.assign(assignment.name, try self.evalExpr(scope, assignment.value));
                return .none;
            },
            .return_stmt => |maybe_expr| {
                if (maybe_expr) |expr| return .{ .returned = try self.evalExpr(scope, expr) };
                return .{ .returned = RuntimeValue{ .number = 0 } };
            },
            .if_stmt => |if_stmt| {
                if ((try self.evalExpr(scope, if_stmt.condition)).truthy()) return self.execBlock(scope, if_stmt.then_block);
                return self.execBlock(scope, if_stmt.else_block);
            },
            .while_stmt => |while_stmt| {
                while ((try self.evalExpr(scope, while_stmt.condition)).truthy()) {
                    const signal = try self.execBlock(scope, while_stmt.body);
                    if (signal == .returned) return signal;
                }
                return .none;
            },
            .expr_stmt => |expr| {
                _ = try self.evalExpr(scope, expr);
                return .none;
            },
        }
    }

    fn evalExpr(self: *Interpreter, scope: *Scope, expr: *const ast.Expr) anyerror!RuntimeValue {
        return switch (expr.*) {
            .number => |n| .{ .number = n },
            .boolean => |b| .{ .boolean = b },
            .variable => |name| try scope.get(name),
            .call => |call| blk: {
                var evaluated_args = try self.allocator.alloc(RuntimeValue, call.args.len);
                defer self.allocator.free(evaluated_args);
                for (call.args, 0..) |arg, i| evaluated_args[i] = try self.evalExpr(scope, arg);
                break :blk try self.callFunction(call.name, evaluated_args);
            },
            .binary => |binary| try self.evalBinary(scope, binary.op, binary.left, binary.right),
        };
    }

    fn evalBinary(self: *Interpreter, scope: *Scope, op: ast.BinaryOp, left: *ast.Expr, right: *ast.Expr) anyerror!RuntimeValue {
        const left_value = try self.evalExpr(scope, left);
        const right_value = try self.evalExpr(scope, right);

        return switch (op) {
            .add => .{ .number = try left_value.asNumber() + try right_value.asNumber() },
            .sub => .{ .number = try left_value.asNumber() - try right_value.asNumber() },
            .mul => .{ .number = try left_value.asNumber() * try right_value.asNumber() },
            .div => .{ .number = try left_value.asNumber() / try right_value.asNumber() },
            .mod => .{ .number = @mod(try left_value.asNumber(), try right_value.asNumber()) },
            .eq => .{ .boolean = compare(left_value, right_value, .eq) },
            .neq => .{ .boolean = compare(left_value, right_value, .neq) },
            .lt => .{ .boolean = try left_value.asNumber() < try right_value.asNumber() },
            .lte => .{ .boolean = try left_value.asNumber() <= try right_value.asNumber() },
            .gt => .{ .boolean = try left_value.asNumber() > try right_value.asNumber() },
            .gte => .{ .boolean = try left_value.asNumber() >= try right_value.asNumber() },
        };
    }
};

fn compare(left: RuntimeValue, right: RuntimeValue, mode: enum { eq, neq }) bool {
    const equal = switch (left) {
        .number => |n| switch (right) { .number => |m| n == m, else => false },
        .boolean => |b| switch (right) { .boolean => |c| b == c, else => false },
    };
    return if (mode == .eq) equal else !equal;
}
