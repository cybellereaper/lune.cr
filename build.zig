const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    const exe = b.addExecutable(.{
        .name = "lune",
        .root_module = b.createModule(.{
            .root_source_file = b.path("src/main.zig"),
            .target = target,
            .optimize = optimize,
        }),
    });
    b.installArtifact(exe);
    const run_cmd = b.addRunArtifact(exe);
    if (b.args) |args| run_cmd.addArgs(args);
    const run_step = b.step("run", "Run the lune CLI");
    run_step.dependOn(&run_cmd.step);

    const lexer_tests = b.addTest(.{
        .root_module = b.createModule(.{
            .root_source_file = b.path("src/lexer.zig"),
            .target = target,
            .optimize = optimize,
        }),
    });

    const run_lexer_tests = b.addRunArtifact(lexer_tests);
    const test_step = b.step("test", "Run lexer unit tests");
    test_step.dependOn(&run_lexer_tests.step);
}
