const io = @import("sys/io.zig");
const string = @import("string.zig");
const extra = @import("extra.zig");
const panic = @import("root.zig").panic;
const stdlib = @import("stdlib.zig");
const syscalls = @import("sys/syscalls.zig");
const errno = @import("sys/errno.zig");
const Errno = @import("sys/errno.zig").Errno;

// TODO: EOF
pub const FILE = extern struct {
    fd: usize,
};
pub export var stdin: FILE = .{ .fd = 0 };
pub export var stdout: FILE = .{ .fd = 1 };

// TODO: work on mode
pub export fn fopen(filename: [*:0]const c_char, mode: [*:0]const c_char) ?*FILE {
    const mode_bytes: [*:0]const u8 = @ptrCast(mode);
    const path: *const u8 = @ptrCast(filename);
    const len = string.strlen(filename);

    var fd = io.open(path, len);
    if (fd == -1) {
        if (mode_bytes[0] == 'w' and errno.errno == @intFromEnum(Errno.NoSuchAFileOrDirectory)) {
            const err = syscalls.create(path, len);
            if (err != 0) return null;

            fd = io.open(path, len);
        } else return null;
    }

    const file = stdlib.zmalloc(FILE).?;
    file.fd = @bitCast(fd);
    return file;
}

pub fn zfopen(filename: [*:0]const u8, mode: [*:0]const u8) ?*FILE {
    return fopen(@ptrCast(filename), @ptrCast(mode));
}

pub export fn fclose(file: *FILE) c_int {
    defer stdlib.free(file);
    if (io.close(@bitCast(file.fd)) < 0) return -1 else return 0;
}

pub export fn fgetc(stream: *FILE) c_int {
    var buffer: u8 = undefined;

    const err = io.read(@bitCast(stream.fd), &buffer, 1);
    if (err < 0) return -1;

    return buffer;
}

pub export fn getc(stream: *FILE) c_int {
    return fgetc(stream);
}

pub export fn fgets(str: [*]c_char, count: c_int, stream: *FILE) ?[*:0]c_char {
    if (count < 1) return null;
    const actual: usize = @intCast(count - 1);

    for (0..actual) |i| {
        const c = getc(stream);

        if (c > 0) str[i] = @intCast(c);
        if (c == '\n' or c == -1) {
            str[i + 1] = 0;
            return @ptrCast(str);
        }
    }

    str[actual + 1] = 0;
    return @ptrCast(str);
}

pub export fn gets_s(str: [*]c_char, count: usize) ?[*:0]c_char {
    if (count < 1) return null;
    const actual: usize = @intCast(count - 1);

    for (0..actual) |i| {
        const c = getc(&stdin);

        if (c == '\n' or c == -1) {
            str[i] = 0;
            return @ptrCast(str);
        }
        str[i] = @intCast(c);
    }

    str[actual + 1] = 0;
    return @ptrCast(str);
}

pub export fn getchar() c_int {
    return fgetc(&stdin);
}

pub export fn fgetline(file: *FILE, len: *usize) ?[*]c_char {
    const ri: isize = @bitCast(file.fd);

    const stat = io.fstat(ri);
    const size = stat.?.size;

    const ptr: ?*u8 = @ptrCast(stdlib.malloc(size));
    if (io.read(ri, ptr orelse return null, size) < 0) return null;
    len.* = size;
    return @ptrCast(ptr);
}

fn wc(c: u8) isize {
    return io.write(1, &c, 1);
}

pub fn zprintf(fmt: [*:0]const u8, ...) callconv(.C) c_int {
    return printf(@ptrCast(fmt));
}

pub export fn printf(fmt: [*:0]const c_char, ...) c_int {
    var arg = @cVaStart();
    var current = fmt;

    while (current[0] != 0) : (current += 1) {
        const start = current;
        var len: usize = 0;

        while (current[0] != '%' and current[0] != 0) {
            current += 1;
            len += 1;
        }

        if (io.write(1, @ptrCast(start), len) < 0) return -1;
        if (current[0] == 0) return 0;

        current += 1;
        switch (current[0]) {
            'd' => {
                var i = @cVaArg(&arg, i32);

                if (i < 0) {
                    i = -i;
                    _ = wc('-');
                }

                var buffer: [10]u8 = [1]u8{0} ** 10;
                _ = extra.itoa(@intCast(i), &buffer, 10);
                if (zprintf("%s", &buffer) < 0) return -1;
            },

            'l' => {
                var i = @cVaArg(&arg, i64);

                if (i < 0) {
                    i = -i;
                    _ = wc('-');
                }

                var buffer: [10]u8 = [1]u8{0} ** 10;
                _ = extra.itoa(@intCast(i), &buffer, 10);
                if (zprintf("%s", &buffer) < 0) return -1;
            },

            'p', 'x' => {
                const i = @cVaArg(&arg, usize);
                var buffer: [10]u8 = [1]u8{0} ** 10;
                _ = extra.itoa(@intCast(i), &buffer, 16);

                if (zprintf("0x%s", &buffer) < 0) return -1;
            },

            's' => {
                const str = @cVaArg(&arg, [*:0]const c_char);
                const strlen = string.strlen(str);
                if (zprintf("%.*s", strlen, str) < 0) return -1;
            },

            '.' => {
                current += 1;
                switch (current[0]) {
                    '*' => {
                        current += 1;
                        const length = @cVaArg(&arg, usize);
                        switch (current[0]) {
                            's' => {
                                const str = @cVaArg(&arg, [*]const u8);
                                if (io.write(1, @ptrCast(str), length) > 0) return -1;
                            },
                            else => {},
                        }
                    },
                    else => {},
                }
            },
            else => continue,
        }
    }

    return 0;
}
