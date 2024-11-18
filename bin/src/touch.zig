const libc = @import("libc");
const File = libc.stdio.File;
const printf = libc.stdio.zprintf;

pub fn main() !void {
    var args = libc.sys.args();
    if (args.count() < 2) {
        try printf("expected at least the file name to touch!\n", .{});
        return error.NotEnoughArguments;
    }

    const filename = args.nth(1).?;
    const file = File.open(filename, .{ .read = true }) catch |err|
        switch (err) {
        error.NoSuchAFileOrDirectory => try File.open(filename, .{ .write = true }),
        else => {
            return err;
        },
    };

    file.close();
}

comptime {
    _ = libc;
}
