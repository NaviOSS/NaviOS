pub const raw = @import("raw.zig");
const syscalls = @import("syscalls.zig");
const errno = @import("errno.zig");
pub export fn sysinfo() ?*raw.SysInfo {
    var info: raw.SysInfo = undefined;
    const err = syscalls.info(&info);
    if (err != 0) {
        errno.errno = @truncate(err);
        return null;
    }

    return &info;
}

pub fn zsysinfo() !raw.SysInfo {
    const ptr = sysinfo() orelse return errno.geterr();
    return ptr.*;
}

pub export fn pcollect(ptr: [*]raw.ProcessInfo, len: usize) isize {
    const err = syscalls.pcollect(@ptrCast(ptr), len);
    if (err == 1) return 1;

    if (err != 0) {
        errno.errno = @truncate(err);
        return -1;
    }
    return 0;
}
/// collects as much processes information as possible in `processes`, returns wether or not it collected all the processes (aka wethere or not buffer were big enough to hold exactly all the processes)
pub fn zpcollect(processes: []raw.ProcessInfo) !bool {
    const results = pcollect(processes.ptr, processes.len);
    if (results == -1) return errno.geterr();
    if (results == 1) return false;
    return true;
}
