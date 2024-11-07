const syscalls = @import("syscalls.zig");
const errors = @import("errno.zig");
const seterr = errors.seterr;

pub fn zsbrk(amount: isize) errors.Error!*anyopaque {
    const brea = syscalls.sbrk(amount);
    if (brea == null)
        return error.OutOfMemory;

    return @ptrCast(brea);
}

pub export fn sbrk(amount: isize) ?*anyopaque {
    return zsbrk(amount) catch |err| {
        seterr(err);
        return null;
    };
}
