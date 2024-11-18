const libc = @import("libc");
const File = libc.stdio.File;
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

    const file = try File.open(filename, .{ .write = true });
    defer file.close();

    const writer = file.writer();
    try writer.write(data);
}
comptime {
    _ = libc;
}
