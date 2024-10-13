const libc = @import("libc");
const printf = libc.stdio.zprintf;

pub fn main() !void {
    var args = libc.sys.args();
    if (args.count() < 2) {
        try printf("expected at least one argument to echo...\n", .{});
        return error.NotEnoughArguments;
    }

    _ = args.next();
    while (args.next()) |arg| {
        try printf("%s", .{arg.ptr});
    }
    try printf("\n", .{});
}

comptime {
    _ = libc;
}
