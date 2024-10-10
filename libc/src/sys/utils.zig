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
