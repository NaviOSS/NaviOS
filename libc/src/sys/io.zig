const syscalls = @import("syscalls.zig");
const errors = @import("errno.zig");
const stdio = @import("../stdio.zig");
pub const raw = @import("raw.zig");

pub fn open(path: *const u8, len: usize) isize {
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

pub export fn diriter_next(diriter: isize) ?*raw.DirEntry {
    var entry: raw.DirEntry = undefined;
    const err = syscalls.diriter_next(@bitCast(diriter), &entry);
    if (err != 0) {
        errors.errno = @truncate(err);
        return null;
    }

    if (entry.name_length == 0 and entry.size == 0 and entry.kind == 0) {
        return null;
    }
    return &entry;
}

pub export fn fstat(ri: isize) ?*raw.DirEntry {
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

pub export fn create(path: *const u8, len: usize) isize {
    const err = syscalls.create(path, len);
    if (err != 0) {
        errors.errno = @truncate(err);
        return -1;
    }

    return 0;
}

pub export fn createdir(path: *const u8, len: usize) isize {
    const err = syscalls.createdir(path, len);
    if (err != 0) {
        errors.errno = @truncate(err);
        return -1;
    }

    return 0;
}
pub fn zopen(path: []const u8) errors.Error!isize {
    const fd = open(@ptrCast(path.ptr), path.len);
    if (fd == -1) return errors.geterr();
    return fd;
}

pub fn zclose(fd: isize) errors.Error!void {
    const err = close(fd);
    if (err == -1) return errors.geterr();
}

pub fn zdiriter_open(dir: isize) errors.Error!isize {
    const ri = diriter_open(dir);
    if (ri == -1) return errors.geterr();
    return ri;
}

pub fn zdiriter_close(diriter: isize) errors.Error!void {
    const err = diriter_close(diriter);
    if (err == -1) return errors.geterr();
}

pub fn zdiriter_next(diriter: isize) ?raw.DirEntry {
    const entry = diriter_next(diriter) orelse return null;
    return entry.*;
}

pub fn zfstat(ri: isize) errors.Error!raw.DirEntry {
    const stat = fstat(ri) orelse return errors.geterr();
    return stat.*;
}

pub fn zread(fd: isize, buffer: []u8) errors.Error!usize {
    const bytes_read = read(fd, @ptrCast(buffer.ptr), buffer.len);
    if (bytes_read == -1) return errors.geterr();
    return @bitCast(bytes_read);
}

pub fn zwrite(fd: isize, buffer: []const u8) errors.Error!usize {
    const bytes_wrote = write(fd, @ptrCast(buffer.ptr), buffer.len);
    if (bytes_wrote == -1) return errors.geterr();
    return @bitCast(bytes_wrote);
}

pub fn zcreate(path: []const u8) errors.Error!void {
    const err = create(@ptrCast(path.ptr), path.len);
    if (err == -1) return errors.geterr();
}

pub fn zcreatedir(path: []const u8) errors.Error!void {
    const err = createdir(@ptrCast(path.ptr), path.len);
    if (err == -1) return errors.geterr();
}

pub export fn chdir(path: [*]const u8, path_len: usize) isize {
    const err = syscalls.chdir(path, path_len);
    if (err != 0) {
        errors.errno = @truncate(err);
        return -1;
    }
    return 0;
}

pub export fn getcwd(ptr: [*]const u8, len: usize) isize {
    var dest_len: usize = undefined;
    const err = syscalls.getcwd(ptr, len, &dest_len);
    if (err != 0) {
        errors.errno = @truncate(err);
        return -1;
    }
    return @bitCast(dest_len);
}

pub fn zgetcwd(buffer: []u8) errors.Error!usize {
    const len = getcwd(@ptrCast(buffer.ptr), buffer.len);
    if (len == -1) return errors.geterr();
    return @bitCast(len);
}

pub fn zchdir(path: []const u8) errors.Error!void {
    const err = chdir(@ptrCast(path.ptr), path.len);
    if (err == -1) return errors.geterr();
}
