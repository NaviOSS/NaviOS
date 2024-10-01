const syscalls = @import("sys/syscalls.zig");

comptime {
    _ = @import("sys/root.zig");
    _ = @import("ctype.zig");
    _ = @import("string.zig");
}

export fn exit() void {
    syscalls.exit();
    asm volatile ("hlt");
}
