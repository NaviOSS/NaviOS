const io = @import("sys/io.zig");
const string = @import("string.zig");
const extra = @import("extra.zig");
const panic = @import("root.zig").panic;
const stdlib = @import("stdlib.zig");
const syscalls = @import("sys/syscalls.zig");
const errors = @import("sys/errno.zig");
const geterr = errors.geterr;
const seterr = errors.seterr;

pub const ModeFlags = packed struct {
    read: bool = false,
    write: bool = false,
    append: bool = false,
    extended: bool = false,
    access_flag: bool = false,

    _padding: u3 = 0,
    pub fn from_cstr(cstr: [*:0]const c_char) ?@This() {
        var bytes: [*:0]const u8 = @ptrCast(cstr);
        var self: ModeFlags = .{};

        while (bytes[0] != 0) : (bytes += 1) {
            const byte = bytes[0];
            switch (byte) {
                'w' => self.write = true,
                'a' => self.append = true,
                'r' => self.read = true,
                '+' => self.extended = true,
                'x' => self.access_flag = true,
                else => return null,
            }
        }

        return self;
    }
};

// TODO: EOF
pub const FILE = extern struct {
    fd: isize,
    mode: ModeFlags,
};
pub export var stdin: FILE = .{ .fd = 0, .mode = .{ .read = true } };
pub export var stdout: FILE = .{ .fd = 1, .mode = .{ .write = true } };

pub fn zfopen(filename: []const u8, mode: ModeFlags) errors.Error!*FILE {
    const fd = io.zopen(filename) catch |err| blk: {
        switch (err) {
            error.NoSuchAFileOrDirectory => if (mode.write or mode.append) {
                try io.zcreate(filename);
                break :blk try io.zopen(filename);
            } else return err,
            else => return err,
        }
    };

    if (mode.write) {
        if (mode.access_flag) {
            return error.AlreadyExists;
        }
        _ = try io.zwrite(fd, "");
    }

    const file = stdlib.zmalloc(FILE).?;
    file.fd = fd;
    file.mode = mode;
    return file;
}

pub export fn fopen(filename: [*:0]const c_char, mode: [*:0]const c_char) ?*FILE {
    const path: [*:0]const u8 = @ptrCast(filename);
    const len = string.strlen(filename);
    const modeflags = ModeFlags.from_cstr(mode) orelse {
        seterr(error.InvaildStr);
        return null;
    };

    return zfopen(path[0..len], modeflags) catch |err| {
        seterr(err);
        return null;
    };
}

pub fn zfclose(file: *FILE) !void {
    defer stdlib.free(file);
    try io.zclose(file.fd);
}

pub export fn fclose(file: *FILE) c_int {
    zfclose(file) catch |err| {
        seterr(err);
        return -1;
    };
    return 0;
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

pub fn zprintf(fmt: [*:0]const u8, args: anytype) !void {
    if (@call(.auto, uprintf, .{fmt} ++ args) == -1) return geterr();
}

pub fn uprintf(fmt: [*:0]const u8, ...) callconv(.C) c_int {
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
                if (uprintf("%s", &buffer) < 0) return -1;
            },

            'l' => {
                var i = @cVaArg(&arg, i64);

                if (i < 0) {
                    i = -i;
                    _ = wc('-');
                }

                var buffer: [10]u8 = [1]u8{0} ** 10;
                _ = extra.itoa(@intCast(i), &buffer, 10);
                if (uprintf("%s", &buffer) < 0) return -1;
            },

            'p', 'x' => {
                const i = @cVaArg(&arg, usize);
                var buffer: [10]u8 = [1]u8{0} ** 10;
                _ = extra.itoa(@intCast(i), &buffer, 16);

                if (uprintf("0x%s", &buffer) < 0) return -1;
            },

            's' => {
                const str = @cVaArg(&arg, [*:0]const c_char);
                const strlen = string.strlen(str);
                if (uprintf("%.*s", strlen, str) == -1) return -1;
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
