const libc = @import("libc");
const sysinfo = libc.sys.utils.zsysinfo;
const zalloc = libc.stdlib.zalloc;
const pcollect = libc.sys.utils.zpcollect;
const printf = libc.stdio.zprintf;

pub fn main() !void {
    const info = try sysinfo();
    const processes = zalloc(libc.sys.raw.ProcessInfo, info.processes_count).?;

    _ = try pcollect(processes);

    try printf("name:  pid  ppid\n", .{});
    for (processes) |process| {
        try printf("\x1B[38;2;0;255;0m%s\x1B[0m:  %d  %d\n", .{ &process.name, process.pid, process.ppid });
    }
}

comptime {
    _ = libc;
}
