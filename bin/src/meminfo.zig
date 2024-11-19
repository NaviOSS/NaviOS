const libc = @import("libc");
const sysinfo = libc.sys.utils.zsysinfo;
const printf = libc.stdio.zprintf;

const Mode = enum {
    Bytes,
    KiB,
    MiB,
    Verbose,
};
pub fn main() !void {
    const info = try sysinfo();
    const mem_ava: usize = info.total_mem - info.used_mem;

    var mode: Mode = .Verbose;
    var args = libc.sys.args();

    while (args.next()) |arg| {
        if (libc.extra.eql(u8, arg, "-b")) {
            mode = .Bytes;
        } else if (libc.extra.eql(u8, arg, "-k")) {
            mode = .KiB;
        } else if (libc.extra.eql(u8, arg, "-m")) {
            mode = .MiB;
        }
    }

    switch (mode) {
        .Bytes => {
            try printf("%luB/%luB\n", .{ info.used_mem, info.total_mem });
        },
        .KiB => {
            try printf("%luKiB/%luKiB\n", .{ info.used_mem / 1024, info.total_mem / 1024 });
        },
        .MiB => {
            try printf("%luMiB/%luMiB\n", .{ info.used_mem / 1024 / 1024, info.total_mem / 1024 / 1024 });
        },

        .Verbose => {
            try printf("memory info:\n", .{});
            try printf("%luB used of %luB, %luB usable\n", .{ info.used_mem, info.total_mem, mem_ava });

            try printf("%luKiBs used of %luKiBs, %luKiBs usable\n", .{ info.used_mem / 1024, info.total_mem / 1024, mem_ava / 1024 });

            try printf("%luMiBs used of %luMiBs, %luMiBs usable\n", .{ info.used_mem / 1024 / 1024, info.total_mem / 1024 / 1024, mem_ava / 1024 / 1024 });
        },
    }
}

comptime {
    _ = libc;
}
