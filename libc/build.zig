const std = @import("std");

// Although this function looks imperative, note that its job is to
// declaratively construct a build graph that will be executed by an external
// runner.
pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});

    // Standard optimization options allow the person running `zig build` to select
    // between Debug, ReleaseSafe, ReleaseFast, and ReleaseSmall. Here we do not
    // set a preferred release mode, allowing the user to decide how to optimize.
    const optimize = b.standardOptimizeOption(.{});

    const freetarget = b.resolveTargetQuery(std.Target.Query{
        .abi = .none,
        .os_tag = .freestanding,
        .ofmt = .elf,
        .cpu_arch = .x86_64,
    });

    const lib = b.addStaticLibrary(.{
        .name = "libc",
        // In this case the main source file is merely a path, however, in more
        // complicated build scripts, this could be a generated file.
        .root_source_file = b.path("src/root.zig"),
        .target = freetarget,
        .optimize = optimize,
        .link_libc = false,
    });

    const lib_check = b.addStaticLibrary(.{
        .name = "libc",
        // In this case the main source file is merely a path, however, in more
        // complicated build scripts, this could be a generated file.
        .root_source_file = b.path("src/root.zig"),
        .target = freetarget,
        .optimize = optimize,
    });

    // This declares intent for the library to be installed into the standard
    // location when the user invokes the "install" step (the default step when
    // running `zig build`).
    b.installArtifact(lib);

    const headergen = b.addExecutable(.{
        .root_source_file = b.path("headergen.zig"),
        .name = "headergen",
        .link_libc = false,
        .target = target,
        .optimize = optimize,
    });

    const headergen_check = b.addExecutable(.{ .root_source_file = b.path("headergen.zig"), .name = "headergen", .link_libc = true, .target = b.host, .optimize = optimize });

    b.installArtifact(headergen);
    const check = b.step("check", "checks if libc compiles");
    check.dependOn(&lib_check.step);
    check.dependOn(&headergen_check.step);

    const headergen_run_cwd = b.addRunArtifact(headergen);
    headergen_run_cwd.step.dependOn(b.getInstallStep());

    const headergen_step = b.step("headergen", "generates the headers");
    headergen_step.dependOn(&headergen_run_cwd.step);
}
