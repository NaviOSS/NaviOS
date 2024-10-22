const libc = @import("libc");
const printf = libc.stdio.zprintf;
const getline = libc.stdio.zgetline;

pub fn main() !void {
    while (true) {
        try printf(">> ", .{});
        const line = try getline();
        defer libc.stdlib.free(line.ptr);
        try printf("%.*s", .{ line.len, line.ptr });
    }
}

comptime {
    _ = libc;
}
