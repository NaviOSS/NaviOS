const libc = @import("libc");
const printf = libc.stdio.zprintf;
const Errno = libc.sys.errno;
const Dir = libc.dirent.DIR;
const eql = libc.extra.eql;
pub fn main() !void {
    var args = libc.sys.args();
    const cwd = try Dir.open(".");
    defer cwd.close();

    var raw_output = false;
    while (args.next()) |arg| {
        if (eql(u8, arg, "--raw")) {
            raw_output = true;
        }
    }

    while (cwd.next()) |ent| {
        if (!raw_output) {
            if (ent.kind == 1)
                try printf("\x1B[38;2;0;100;255m%.*s\n\x1B[0m", .{ ent.name_length, &ent.name })
            else if (ent.kind == 2)
                try printf("\x1B[38;2;255;0;0m%.*s\n\x1B[0m", .{ ent.name_length, &ent.name })
            else
                try printf("%.*s\n", .{ ent.name_length, &ent.name });
        } else try printf("%.*s\n", .{ ent.name_length, &ent.name });
    }
}

comptime {
    _ = libc;
}
