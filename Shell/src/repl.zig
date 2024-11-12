const libc = @import("libc");
const Token = @import("Lexer.zig").Token;
const alloc = libc.stdlib.zalloc;
const free = libc.stdlib.free;
const zpspawn = libc.sys.utils.zpspwan;
const Slice = libc.sys.raw.Slice;
const Error = libc.sys.errno.Error;
const eql = @import("utils.zig").eql;
const environment = @import("environment.zig");
const ArrayList = @import("utils.zig").ArrayList;

const ExecuteBuiltin = @import("builtin.zig").executeBuiltin;

fn spawn(name: []const u8, argv: []const Slice(u8)) Error!u64 {
    var path_var = try environment.get_path();
    defer path_var.deinit();

    for (path_var.items) |path| {
        var it = try libc.dirent.zopendir(path);
        defer it.close();

        while (it.next()) |entry| {
            const entry_name = entry.name[0..entry.name_length];

            if (eql(u8, entry_name, name)) {
                var full_path = try ArrayList(u8).init();
                defer full_path.deinit();

                try full_path.set_len(path.len + 1 + entry_name.len);

                libc.string.zmemcpy(u8, full_path.items, path);
                full_path.items[path.len] = '/';
                libc.string.zmemcpy(u8, full_path.items[path.len + 1 ..], entry_name);

                const pid = zpspawn(full_path.items, argv, name);
                return pid;
            }
        }
    }

    return error.NoSuchAFileOrDirectory;
}

fn wait(pid: u64) usize {
    return libc.syscalls.wait(pid);
}

pub fn repl(tokens: []const Token) Error!usize {
    if (tokens.len == 0) return 0;

    const argv = try alloc(Slice(u8), tokens.len);
    defer free(argv.ptr);

    for (tokens, 0..) |token, i| {
        const string = token.asString();

        argv[i] = .{ .ptr = string.ptr, .len = string.len };
    }

    const name = argv[0];
    const results = ExecuteBuiltin(name, argv) orelse {
        const pid = try spawn(name.ptr[0..name.len], argv);
        return wait(pid);
    };
    return results;
}
