const libc = @import("libc");
const printf = libc.stdio.zprintf;

pub fn main() !void {
    var args = libc.sys.args();
    if (args.count() < 2) {
        try printf("expected at least the file name to touch!\n", .{});
        return error.NotEnoughArguments;
    }

    const filename = args.nth(1).?;
    const file = libc.stdio.zfopen(filename, .{ .read = true }) catch |err|
        switch (err) {
        error.NoSuchAFileOrDirectory => try libc.stdio.zfopen(filename, .{ .write = true }),
        else => {
            try printf("got error %s\n", .{@errorName(err).ptr});
            return;
        },
    };

    try libc.stdio.zfclose(file);
}

comptime {
    _ = libc;
}
