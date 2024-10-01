//! forces the zig compiler to compile:
pub const io = @import("io.zig");
pub const errno = @import("errno.zig");
pub const raw = @import("raw.zig");

comptime {
    _ = io;
    _ = errno;
    _ = raw;
}
