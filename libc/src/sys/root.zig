//! forces the zig compiler to compile:
pub const io = @import("io.zig");
pub const errno = @import("errno.zig");
pub const raw = @import("raw.zig");
pub const mem = @import("mem.zig");
pub const utils = @import("utils.zig");

comptime {
    _ = io;
    _ = errno;
    _ = raw;
    _ = mem;
    _ = utils;
}

const private = @import("../private.zig");

pub const ArgsIterator = struct {
    at: usize = 0,
    argc: usize,
    argv: [*]const *const raw.OsStr,

    pub fn next(self: *@This()) ?[]const u8 {
        defer self.at += 1;
        return self.nth(self.at);
    }

    pub fn nth(self: *const @This(), n: usize) ?[]const u8 {
        if (n >= self.argc) {
            return null;
        }

        var arg = @constCast(self.argv[n]);
        const data: [*]const u8 = @ptrCast(arg.data());
        return data[0..arg.len];
    }

    pub fn count(self: *const @This()) usize {
        return self.argc;
    }
};

pub fn args() ArgsIterator {
    return .{ .argc = private.__lib__argc_get(), .argv = private.__lib__argv_get() };
}
