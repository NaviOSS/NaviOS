const libc = @import("libc");
const printf = libc.stdio.zprintf;
pub const panic = libc.panic;
pub fn main() !void {
    const args = libc.sys.args();
    if (args.count() < 2) {
        try printf("expected filename to cat\n", .{});
        return error.NotEnoughArguments;
    }

    const filename = args.nth(1).?;
    const file = try libc.stdio.zfopen(filename, .{ .read = true });

    var len: usize = 0;
    const data = libc.stdio.fgetline(file, &len);
    try printf("%.*s\n", .{ len, data });

    try libc.stdio.zfclose(file);
}

comptime {
    _ = libc;
}
