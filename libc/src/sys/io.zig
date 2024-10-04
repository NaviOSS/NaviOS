const syscalls = @import("syscalls.zig");
const errors = @import("errno.zig");
pub const raw = @import("raw.zig");

pub export fn open(path: *const u8, len: usize) isize {
    var fd: usize = undefined;

    const err = syscalls.open(path, len, &fd);
    if (err != 0) {
        errors.errno = @truncate(err);
        return -1;
    }
    return @bitCast(fd);
}

pub export fn close(fd: isize) isize {
    const err = syscalls.close(@bitCast(fd));
    if (err != 0) {
        errors.errno = @truncate(err);
        return -1;
    }
    return 0;
}

pub export fn diriter_open(dir: isize) isize {
    var diriter: usize = undefined;
    const err = syscalls.diriter_open(@bitCast(dir), &diriter);

    if (err != 0) {
        errors.errno = @truncate(err);
        return -1;
    }

    return @bitCast(diriter);
}

pub export fn diriter_close(diriter: isize) isize {
    const err = syscalls.diriter_close(@bitCast(diriter));
    if (err != 0) {
        errors.errno = @truncate(err);
        return -1;
    }

    return 0;
}

pub export fn diriter_next(diriter: isize) ?*const raw.DirEntry {
    var entry: raw.DirEntry = undefined;
    const err = syscalls.diriter_next(@bitCast(diriter), &entry);

    if (err != 0) {
        errors.errno = @truncate(err);
        return null;
    }

    return &entry;
}

pub export fn fstat(ri: isize) ?*const raw.DirEntry {
    var entry: raw.DirEntry = undefined;
    const err = syscalls.fstat(@bitCast(ri), &entry);

    if (err != 0) {
        errors.errno = @truncate(err);
        return null;
    }

    return &entry;
}

pub export fn read(fd: isize, ptr: *u8, size: usize) isize {
    var bytes_read: usize = undefined;

    const err = syscalls.read(@bitCast(fd), ptr, size, &bytes_read);
    if (err != 0) {
        errors.errno = @truncate(err);
        return -1;
    }
    return @bitCast(bytes_read);
}

pub export fn write(fd: isize, ptr: *const u8, size: usize) isize {
    const err = syscalls.write(@bitCast(fd), ptr, size);
    if (err != 0) {
        errors.errno = @truncate(err);
        return -1;
    }
    return 0;
}
pub inline fn zopen(path: []const u8) isize {
    return open(path.ptr, path.len);
}
