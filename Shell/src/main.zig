const libc = @import("libc");
const printf = libc.stdio.zprintf;
const getline = libc.stdio.zgetline;
const Lexer = @import("Lexer.zig");
const repl = @import("repl.zig");
const Error = libc.sys.errno.Error;

pub const panic = libc.panic;
const ArrayList = @import("utils.zig").ArrayList;

pub fn main() Error!void {
    var ret: u64 = 0;
    while (true) {
        const cwd_buffer = try libc.stdlib.zalloc(u8, 1024);
        defer libc.stdlib.free(cwd_buffer.ptr);

        const cwd_len = try libc.sys.io.zgetcwd(cwd_buffer);
        try printf("\x1B[38;2;255;0;193m%.*s\x1B[0m ", .{ cwd_len, cwd_buffer.ptr });

        if (ret != 0) {
            try printf("\x1B[38;2;255;0;0m[%l]\x1B[0m ", .{ret});
        }

        try printf("# ", .{});

        const line = try getline();
        defer libc.stdlib.free(line.ptr);

        var tokens = try ArrayList(Lexer.Token).init();
        defer tokens.deinit();

        var lexer = Lexer.init(line);
        while (lexer.next()) |token| {
            try tokens.append(token);
        }

        const name = tokens.items[0].asString();
        ret = repl.repl(tokens.items) catch |err| blk: {
            const err_name = @errorName(err);
            try printf("failed to execute %.*s, error: %.*s\n", .{ name.len, name.ptr, err_name.len, err_name.ptr });
            break :blk 0;
        };
    }
}

comptime {
    _ = libc;
}
