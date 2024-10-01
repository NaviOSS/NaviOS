pub const syscalls = @import("sys/syscalls.zig");
pub const sys_root = @import("sys/root.zig");
pub const ctype = @import("ctype.zig");
pub const string = @import("string.zig");

comptime {
    _ = syscalls;
    _ = sys_root;
    _ = ctype;
    _ = string;
}

export fn exit() void {
    syscalls.exit();
    asm volatile ("hlt");
}
