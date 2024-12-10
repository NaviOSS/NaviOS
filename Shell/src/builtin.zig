const libc = @import("libc");
const eql = @import("utils.zig").eql;
const Slice = libc.sys.raw.Slice;

pub fn exit() noreturn {
    libc.exit(1);
}

pub fn cd(argv: []const Slice(u8)) u64 {
    if (argv.len < 2) return @intFromError(libc.sys.errno.Error.NotEnoughArguments);
    const path = argv[1];
    libc.sys.io.zchdir(path.ptr[0..path.len]) catch |err| return @intFromError(err);
    return 0;
}

pub fn help() void {
    libc.stdio.zprintf(
        \\to scroll up use PageUp, to scroll down use PageDown
        \\### Basic builtin commands list:
        \\
    , .{}) catch {};
    for (BuiltinFunctions) |function| {
        libc.stdio.zprintf("- %.*s\n", .{ function.len, function.ptr }) catch {};
    }
}

pub fn shutdown() noreturn {
    libc.syscalls.shutdown();
}

pub fn reboot() noreturn {
    libc.syscalls.reboot();
}

pub fn clear() void {
    libc.stdio.zprintf("\x1B[2J\x1B[H", .{}) catch {};
}

pub fn getBuitlinFunctions() []const []const u8 {
    const self = @This();
    const info = @typeInfo(self);
    const decls = info.Struct.decls;
    comptime var functions: []const []const u8 = &[_][]const u8{};

    inline for (decls) |decl| {
        const field = @TypeOf(@field(self, decl.name));

        if (@typeInfo(field) == .Fn) {
            functions = functions ++ &[_][]const u8{decl.name};
        }
    }

    return functions;
}

const BuiltinFunctions = getBuitlinFunctions();

pub fn executeBuiltin(name: Slice(u8), argv: []const Slice(u8)) ?u64 {
    inline for (BuiltinFunctions) |function| {
        const func = @field(@This(), function);
        const ty = @TypeOf(func);
        const info = @typeInfo(ty);

        if (eql(u8, function, name.ptr[0..name.len])) {
            const args = switch (info.Fn.params.len) {
                0 => .{},
                1 => .{argv},
                else => return null,
            };

            switch (info.Fn.return_type.?) {
                void => {
                    @call(.auto, func, args);
                    return 0;
                },
                u64, noreturn => return @call(.auto, func, .{} ++ args),
                else => {},
            }
        }
    }

    return null;
}
