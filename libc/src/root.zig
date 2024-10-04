const builtin = @import("std").builtin;
const syscalls = @import("sys/syscalls.zig");
pub const sys = @import("sys/root.zig");
pub const ctype = @import("ctype.zig");
pub const string = @import("string.zig");
pub const stdio = @import("stdio.zig");
pub const extra = @import("extra.zig");

comptime {
    _ = sys;
    _ = ctype;
    _ = string;
    _ = stdio;
    _ = extra;
}

export fn exit() noreturn {
    syscalls.exit();
    while (true) {
        asm volatile ("hlt");
    }
}

pub fn panic(msg: []const u8, error_return_trace: ?*builtin.StackTrace, return_addr: ?usize) noreturn {
    @setCold(true);
    const at = return_addr orelse @returnAddress();
    _ = stdio.zprintf("panic: %.*s at %p\n", msg.len, msg.ptr, at);

    if (error_return_trace) |trace| {
        _ = stdio.zprintf("trace:\n");
        const addresses = trace.instruction_addresses;

        for (addresses) |address| {
            _ = stdio.zprintf("  <%p>\n", address);
        }
    }

    exit();
}

extern fn main(argc: usize, argv: **sys.raw.OsStr) i32;

fn _start() callconv(.Naked) noreturn {
    asm volatile (
        \\ movq $0, %rbp
        \\ push %rbp
        \\ push %rbp
        \\ push %rdi
        \\ push %rsi
        \\ pop %rsi
        \\ pop %rdi
        \\ call main
        \\ call exit
        \\ hlt
    );
}

// we cannot export start directly to avoid problems with headergen
comptime {
    if (@import("builtin").os.tag == .freestanding) {
        @export(_start, .{ .name = "_start" });
    }
}
