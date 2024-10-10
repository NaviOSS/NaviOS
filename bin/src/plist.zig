const libc = @import("libc");
const sysinfo = libc.sys.utils.sysinfo;
const zalloc = libc.stdlib.zalloc;
const pcollect = libc.syscalls.pcollect;
const printf = libc.stdio.zprintf;

export fn main() i32 {
    const info = sysinfo().?.*;
    const processes = zalloc(libc.sys.raw.ProcessInfo, info.processes_count).?;

    if (libc.syscalls.pcollect(@ptrCast(processes.ptr), info.processes_count) != 0) return -1;

    _ = printf("name:  pid  ppid\n");
    for (processes) |process| {
        _ = printf("\x1B[38;2;0;255;0m%s\x1B[0m:  %d  %d\n", &process.name, process.pid, process.ppid);
    }
    return 0;
}
