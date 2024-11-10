const libc = @import("libc");
const Token = @import("Lexer.zig").Token;
const alloc = libc.stdlib.zalloc;
const free = libc.stdlib.free;
const zspawn = libc.sys.utils.zspwan;
const Slice = libc.sys.raw.Slice;
const Error = libc.sys.errno.Error;

const ExecuteBuiltin = @import("builtin.zig").executeBuiltin;

fn spawn(name: []const u8, argv: []const Slice(u8)) Error!u64 {
    const file = try libc.stdio.zfopen(name, .{ .read = true });
    defer libc.stdio.zfclose(file) catch unreachable;

    const stat = try libc.sys.io.zfstat(file.fd);
    const size = stat.size;

    const buffer = try alloc(u8, size);
    defer free(buffer.ptr);

    _ = try libc.sys.io.zread(file.fd, buffer);
    return zspawn(buffer, argv, name);
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
