const io = @import("sys/io.zig");
const string = @import("string.zig");
const extra = @import("extra.zig");
const panic = @import("root.zig").panic;
const stdlib = @import("stdlib.zig");
const syscalls = @import("sys/syscalls.zig");
const errors = @import("sys/errno.zig");
const geterr = errors.geterr;
const seterr = errors.seterr;
const EOF: u8 = 255;

const VaList = @import("std").builtin.VaList;

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
// TODO: actually rwlock the file
pub const Writer = struct {
    file: *FILE,
    fn check(self: *const Writer) errors.Error!void {
        if (!self.file.mode.write) return error.MissingPermissions;
    }

    pub fn write(self: *const Writer, buf: []const u8) errors.Error!void {
        try self.check();
        _ = try io.zwrite(self.file.fd, buf);
    }

    pub fn writeByte(self: *const Writer, c: u8) errors.Error!void {
        return self.write(&[1]u8{c});
    }

    /// writes an argument to the file and formats it based on the provided format string
    /// the format string should be only the format specifier
    pub fn writeArg(self: *const Writer, comptime fmt: []const u8, arg: anytype) !void {
        var value = arg;
        const ty = @TypeOf(arg);

        switch (fmt[0]) {
            'd' => {
                switch (ty) {
                    i32, i64, isize => {
                        if (value < 0) {
                            try self.writeByte('-');
                            value = -value;
                        }

                        var buffer: [20:0]u8 = [1:0]u8{0} ** 20;
                        _ = extra.itoa(@intCast(arg), &buffer, 10);

                        const ptr: [*:0]const u8 = @ptrCast(&buffer);
                        try self.writeArg("s", ptr);
                    },

                    else => @compileError("invaild type for fmt 'd' " ++ @typeName(ty)),
                }
            },

            'u' => {
                switch (ty) {
                    u32, u64, usize => {
                        var buffer: [20:0]u8 = [1:0]u8{0} ** 20;
                        _ = extra.itoa(@intCast(arg), &buffer, 10);

                        const ptr: [*:0]const u8 = @ptrCast(&buffer);
                        try self.writeArg("s", ptr);
                    },

                    else => @compileError("invaild type for fmt 'u' " ++ @typeName(ty)),
                }
            },

            'x' => {
                switch (ty) {
                    u32, u64, usize => {
                        var buffer: [16:0]u8 = [1:0]u8{0} ** 16;
                        _ = extra.itoa(@intCast(arg), &buffer, 16);
                        const ptr: [*:0]const u8 = @ptrCast(&buffer);
                        try self.writeArg("s", ptr);
                    },
                    else => @compileError("invaild type for fmt 'x' " ++ @typeName(ty)),
                }
            },

            's' => {
                switch (ty) {
                    []const u8 => {
                        try self.write(arg);
                    },
                    [*:0]const u8 => {
                        const len = string.strlen(@ptrCast(value));
                        try self.write(value[0..len]);
                    },

                    else => @compileError("invaild type for fmt 's' " ++ @typeName(ty)),
                }
            },

            else => @compileError("invalid format specifier " ++ fmt),
        }
    }
    /// writes a formatted string to the file
    /// until a format specifier is found, returning the start of the format specifier
    fn traverseFmt(self: *const Writer, fmt: [*:0]const u8) !?[*:0]const u8 {
        var current = fmt;
        var len: usize = 0;
        while (current[0] != '%' and current[0] != 0) {
            current += 1;
            len += 1;
        }

        try self.write(fmt[0..len]);
        if (current[0] == 0) return null;
        return current;
    }

    pub fn writeVarFmt(self: *const Writer, fmt: [*:0]const u8, args: *VaList) errors.Error!void {
        var current = fmt;

        while (current[0] != 0) : (current += 1) {
            current = try self.traverseFmt(current) orelse return;
            current += 1;
            switch (current[0]) {
                'd' => {
                    const i = @cVaArg(args, i32);
                    try self.writeArg("d", i);
                },

                'u' => {
                    const i = @cVaArg(args, u32);
                    try self.writeArg("u", i);
                },

                'l' => {
                    if (current[1] == 'u') {
                        current += 1;
                        const i = @cVaArg(args, u64);
                        try self.writeArg("u", i);
                    } else {
                        const i = @cVaArg(args, i64);
                        try self.writeArg("d", i);
                    }
                },

                'p', 'x' => {
                    const i = @cVaArg(args, usize);
                    try self.writeArg("x", i);
                },

                's' => {
                    const str = @cVaArg(args, [*:0]const u8);
                    try self.writeArg("s", str);
                },

                '.' => {
                    current += 1;
                    switch (current[0]) {
                        '*' => {
                            current += 1;
                            const length = @cVaArg(args, usize);
                            switch (current[0]) {
                                's' => {
                                    const str = @cVaArg(args, [*]const u8);
                                    try self.writeArg("s", str[0..length]);
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
    }

    fn writeVVarFmt(self: *const Writer, fmt: [*:0]const u8, ...) callconv(.C) i32 {
        var arg = @cVaStart();
        self.writeVarFmt(fmt, &arg) catch |err| {
            seterr(err);
            return -1;
        };

        return 0;
    }

    /// writes a args to the file formated based on `fmt`
    /// requires C-style fmt
    pub fn writeFmt(self: *const Writer, fmt: [*:0]const u8, args: anytype) errors.Error!void {
        // const ty = @TypeOf(args);
        // const info = @typeInfo(ty).Struct;
        //
        // inline for (info.fields) |field| {
        //     const value = @field(args, field.name);
        //
        //     c_args = c_args ++ switch (field.type) {
        //         []const u8, [:0]const u8, []u8, [:0]u8 => .{ value.len, value.ptr },
        //         else => .{value},
        //     };
        // }

        if (@call(.auto, Writer.writeVVarFmt, .{ self, fmt } ++ args) != 0) return geterr();
    }
};

pub const Reader = struct {
    file: *FILE,
    fn check(self: *const Reader) errors.Error!void {
        if (!self.file.mode.read) return error.MissingPermissions;
    }

    pub fn read(self: *const Reader, buf: []u8) errors.Error!usize {
        try self.check();
        return io.zread(self.file.fd, buf);
    }

    pub fn readByte(self: *const Reader) errors.Error!u8 {
        var buffer: [1]u8 = undefined;
        const amount = try self.read(&buffer);
        if (amount == 0) return EOF;
        return buffer[0];
    }

    /// reads until the delimiter is found or EOF is reached
    /// output doesn't include the delimiter
    /// the returned buffer is allocated
    pub fn readUntilEOFOrDelimiter(self: *const Reader, delimiter: u8) errors.Error![]u8 {
        try self.check();
        var buf = try stdlib.zalloc(u8, 0);
        var i: usize = 0;

        while (true) : (i += 1) {
            const c = try self.readByte();
            if (c == delimiter or c == EOF) {
                break;
            }

            buf = stdlib.zrealloc(u8, buf, buf.len + 1) orelse return error.OutOfMemory;
            buf[i] = @intCast(c);
        }

        return buf;
    }

    /// reads until EOF is reached
    /// the returned buffer is allocated
    pub fn readUntilEOF(self: *const Reader) errors.Error![]u8 {
        try self.check();
        var buf = try stdlib.zalloc(u8, 0);
        var i: usize = 0;

        while (true) : (i += 1) {
            const c = try self.readByte();
            if (c == EOF) {
                break;
            }

            buf = stdlib.zrealloc(u8, buf, buf.len + 1) orelse return error.OutOfMemory;
            buf[i] = @intCast(c);
        }

        return buf;
    }
};

pub const FILE = extern struct {
    fd: isize,
    mode: ModeFlags,

    pub fn open(filename: []const u8, mode: ModeFlags) errors.Error!*FILE {
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

    pub fn closeChecked(file: *FILE) errors.Error!void {
        defer stdlib.free(file);
        try io.zclose(file.fd);
    }

    pub fn close(file: *FILE) void {
        file.closeChecked() catch unreachable;
    }

    pub fn writer(self: *FILE) Writer {
        return .{ .file = self };
    }

    pub fn reader(self: *FILE) Reader {
        return .{ .file = self };
    }
};

pub const File = FILE;
pub export var stdin: FILE = .{ .fd = 0, .mode = .{ .read = true } };
pub export var stdout: FILE = .{ .fd = 1, .mode = .{ .write = true } };

pub export fn fopen(filename: [*:0]const c_char, mode: [*:0]const c_char) ?*FILE {
    const path: [*:0]const u8 = @ptrCast(filename);
    const len = string.strlen(filename);
    const modeflags = ModeFlags.from_cstr(mode) orelse {
        seterr(error.InvaildStr);
        return null;
    };

    return FILE.open(path[0..len], modeflags) catch |err| {
        seterr(err);
        return null;
    };
}

pub export fn fclose(file: *FILE) c_int {
    FILE.closeChecked(file) catch |err| {
        seterr(err);
        return -1;
    };
    return 0;
}

fn zfgetc(stream: *FILE) errors.Error!u8 {
    const reader = stream.reader();
    return reader.readByte();
}

pub export fn fgetc(stream: *FILE) c_int {
    const c = zfgetc(stream) catch |err| {
        seterr(err);
        return -1;
    };

    return @intCast(c);
}

pub export fn getc(stream: *FILE) c_int {
    return fgetc(stream);
}

// pub export fn fgets(str: [*]c_char, count: c_int, stream: *FILE) ?[*:0]c_char {
//     if (count < 1) return null;
//     const actual: usize = @intCast(count - 1);
//
//     for (0..actual) |i| {
//         const c = getc(stream);
//
//         if (c > 0) str[i] = @intCast(c);
//         if (c == '\n' or c == -1) {
//             str[i + 1] = 0;
//             return @ptrCast(str);
//         }
//     }
//
//     str[actual + 1] = 0;
//     return @ptrCast(str);
// }
//
// pub export fn gets_s(str: [*]c_char, count: usize) ?[*:0]c_char {
//     if (count < 1) return null;
//     const actual: usize = @intCast(count - 1);
//
//     for (0..actual) |i| {
//         const c = getc(&stdin);
//
//         if (c == '\n' or c == -1) {
//             str[i] = 0;
//             return @ptrCast(str);
//         }
//         str[i] = @intCast(c);
//     }
//
//     str[actual + 1] = 0;
//     return @ptrCast(str);
// }

pub export fn getchar() c_int {
    return fgetc(&stdin);
}

fn zfgetline(file: *FILE) ![]u8 {
    var buffer = try file.reader().readUntilEOFOrDelimiter('\n');
    buffer = stdlib.zrealloc(u8, buffer, buffer.len + 2) orelse return error.OutOfMemory;
    buffer[buffer.len - 2] = '\n';
    buffer[buffer.len - 1] = 0;
    return buffer;
}

pub fn zgetline() ![]u8 {
    return stdin.reader().readUntilEOFOrDelimiter('\n');
}

pub export fn fgetline(file: *FILE, len: *usize) ?[*]c_char {
    const slice = zfgetline(file) catch |err| {
        seterr(err);
        return null;
    };
    len.* = slice.len;
    return @ptrCast(slice.ptr);
}

fn wc(c: u8) isize {
    return io.write(1, &c, 1);
}

pub fn zprintf(fmt: [*:0]const u8, args: anytype) !void {
    return stdout.writer().writeFmt(fmt, args);
}

pub export fn printf(fmt: [*:0]const c_char, ...) c_int {
    return stdout.writer().writeVVarFmt(@ptrCast(fmt));
}
