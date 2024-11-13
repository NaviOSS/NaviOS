const libc = @import("libc");
const printf = libc.stdio.zprintf;
pub const panic = libc.panic;

pub fn main() !void {
    const args = libc.sys.args();
    if (args.count() < 3) {
        try printf("expected filename to write to, and data to write\n", .{});
        return error.NotEnoughArguments;
    }
    const filename = args.nth(1).?;
    const data = args.nth(2).?;
    const file = try libc.stdio.zfopen(filename, .{ .write = true });

    _ = try libc.sys.io.zwrite(file.fd, data);

    try libc.stdio.zfclose(file);
}
comptime {
    _ = libc;
}
