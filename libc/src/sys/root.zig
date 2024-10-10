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
