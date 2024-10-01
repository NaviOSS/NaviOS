const syscalls = @import("sys/syscalls.zig");
pub const sys = @import("sys/root.zig");
pub const ctype = @import("ctype.zig");
pub const string = @import("string.zig");

comptime {
    _ = sys;
    _ = ctype;
    _ = string;
}

export fn exit() void {
    syscalls.exit();
    asm volatile ("hlt");
}
