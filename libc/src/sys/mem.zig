const syscalls = @import("syscalls.zig");
const errno = @import("errno.zig");

pub export fn sbrk(amount: isize) ?*void {
    const brea = syscalls.sbrk(amount);
    if (brea == null)
        errno.errno = @intFromEnum(errno.Errno.MMapError);

    return @ptrCast(brea);
}
