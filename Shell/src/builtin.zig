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

        if (info.Fn.return_type == void or info.Fn.return_type == u64 or info.Fn.return_type == noreturn)
            if (eql(u8, function, name.ptr[0..name.len])) {
                if (info.Fn.params.len == 0) return @call(.auto, func, .{});
                if (info.Fn.params.len == 1) return @call(.auto, func, .{argv});
            };
    }

    return null;
}
