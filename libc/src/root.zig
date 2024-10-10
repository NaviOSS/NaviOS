const builtin = @import("std").builtin;
pub const syscalls = @import("sys/syscalls.zig");
pub const sys = @import("sys/root.zig");
pub const ctype = @import("ctype.zig");
pub const string = @import("string.zig");
pub const stdio = @import("stdio.zig");
pub const stdlib = @import("stdlib.zig");
pub const extra = @import("extra.zig");
pub const dirent = @import("dirent.zig");

comptime {
    _ = sys;
    _ = ctype;
    _ = string;
    _ = stdio;
    _ = stdlib;
    _ = extra;
    _ = dirent;
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
    _ = stdio.zprintf("libc panic: %.*s at %p <??>\n", msg.len, msg.ptr, at);

    if (error_return_trace) |trace| {
        _ = stdio.zprintf("trace:\n");
        const addresses = trace.instruction_addresses;

        for (addresses) |address| {
            _ = stdio.zprintf("  <%p>\n", address);
        }
    }

    exit();
}

extern fn main(argc: usize, argv: [*]*c_char) i32;

/// starts as C
fn __libc_c_start() callconv(.Naked) i32 {
    // converting argv to **char
    asm volatile (
        \\ # rax = size + 8 (first argument copying)
        \\ mov %rdi, %rax
        \\ shl $3, %rax
        \\ add $8, %rax
        \\ # we are going to reuse rsi
        \\ mov %rsi, %rdx
        \\ # allocating on the stack
        \\ mov %rsp, %rcx
        \\ sub %rax, %rsp
        \\ mov %rsp, %rsi
        \\ # pushing the return value
        \\ push (%rcx)
        \\ jmp __libc_OsStr_to_cstr
        \\ # reverse looping
        \\ __libc_OsStr_to_cstr:
        \\  sub $8, %rax
        \\  # rsi + rax = rdx + rax
        \\  add %rax, %rsi
        \\  add %rax, %rdx
        \\  # =
        \\  mov (%rdx), %rbx
        \\  # skipping len in OsStr
        \\  add $8, %rbx
        \\  mov %rbx, (%rsi)
        \\  # restore
        \\  sub %rax, %rsi
        \\  sub %rax, %rdx
        \\  # if rax == 0 jmp loop else jmp finish
        \\  test %rax, %rax
        \\  jnz __libc_OsStr_to_cstr
        \\  call main
        \\  ret
    );
}
fn _start() callconv(.Naked) noreturn {
    asm volatile (
        \\ movq $0, %rbp
        \\ push %rbp
        \\ push %rbp
        \\ push %rdi
        \\ push %rsi
        \\ pop %rsi
        \\ pop %rdi
        \\ call __libc_c_start
        \\ call exit
        \\ hlt
    );
}

// we cannot export start directly to avoid problems with headergen
comptime {
    if (@import("builtin").os.tag == .freestanding) {
        @export(_start, .{ .name = "_start" });
        @export(__libc_c_start, .{ .name = "__libc_c_start" });
    }
}
