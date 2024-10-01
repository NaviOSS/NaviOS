const syscalls = @import("syscalls.zig");
const errors = @import("errno.zig");

export fn open(path: *const u8, len: usize) isize {
    var fd: usize = undefined;

    const err = syscalls.open(path, len, &fd);
    if (err != 0) {
        errors.errno = @truncate(err);
        return -1;
    }
    return @bitCast(fd);
}

export fn close(fd: isize) isize {
    const err = syscalls.close(fd);
    if (err != 0) {
        errors.errno = @truncate(err);
        return -1;
    }
    return 0;
}

inline fn zopen(path: []const u8) isize {
    return open(path.ptr, path.len);
}
