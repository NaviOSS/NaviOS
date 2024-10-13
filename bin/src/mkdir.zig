const libc = @import("libc");
const io = libc.sys.io;
const strlen = libc.string.strlen;
const printf = libc.stdio.zprintf;

pub fn main() !void {
    var args = libc.sys.args();
    if (args.count() < 2) {
        try printf("expected at least the name of the directory to make\n", .{});
        return error.NotEnoughArguments;
    }

    const path = args.nth(1).?;
    try io.zcreatedir(path);
}

comptime {
    _ = libc;
}
