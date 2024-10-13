const libc = @import("libc");
const sysinfo = libc.sys.utils.zsysinfo;
const printf = libc.stdio.zprintf;
pub fn main() !void {
    const info = try sysinfo();
    const mem_ava: usize = info.total_mem - info.used_mem;

    try printf("memory info:\n", .{});
    try printf("%dB used of %dB, %dB usable\n", .{ info.used_mem, info.total_mem, mem_ava });

    try printf("%dKiBs used of %dKiBs, %dKiBs usable\n", .{ info.used_mem / 1024, info.total_mem / 1024, mem_ava / 1024 });

    try printf("%dMiBs used of %dMiBs, %dMiBs usable\n", .{ info.used_mem / 1024 / 1024, info.total_mem / 1024 / 1024, mem_ava / 1024 / 1024 });
}

comptime {
    _ = libc;
}
