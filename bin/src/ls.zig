const libc = @import("libc");
const printf = libc.stdio.zprintf;
const Errno = libc.sys.errno;

pub fn main() !void {
    const cwd = try libc.dirent.zopendir(".");
    defer cwd.close();

    while (cwd.next()) |ent| {
        if (ent.kind == 1)
            try printf("\x1B[38;2;255;255;0m%.*s\n\x1B[0m", .{ ent.name_length, &ent.name })
        else if (ent.kind == 2)
            try printf("\x1B[38;2;255;0;0m%.*s\n\x1B[0m", .{ ent.name_length, &ent.name })
        else
            try printf("%.*s\n", .{ ent.name_length, &ent.name });
    }
}

comptime {
    _ = libc;
}
