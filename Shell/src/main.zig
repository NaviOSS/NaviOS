const libc = @import("libc");
const printf = libc.stdio.zprintf;
const getline = libc.stdio.zgetline;
const Lexer = @import("Lexer.zig");
const repl = @import("repl.zig");
pub const panic = libc.panic;
const ArrayList = @import("utils.zig").ArrayList;

pub fn main() !void {
    while (true) {
        try printf(">> ", .{});
        const line = try getline();
        defer libc.stdlib.free(line.ptr);

        var tokens = try ArrayList(Lexer.Token).init();
        defer tokens.deinit();

        var lexer = Lexer.init(line);
        while (lexer.next()) |token| {
            try tokens.append(token);
        }

        const name = tokens.items[0].asString();
        _ = repl.repl(tokens.items) catch |err| {
            const err_name = @errorName(err);
            try printf("failed to execute %.*s, error: %.*s\n", .{ name.len, name.ptr, err_name.len, err_name.ptr });
        };
    }
}

comptime {
    _ = libc;
}
