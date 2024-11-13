const builtin = @import("std").builtin;
const builtin_info = @import("builtin");
pub const syscalls = @import("sys/syscalls.zig");
pub const sys = @import("sys/root.zig");
pub const ctype = @import("ctype.zig");
pub const string = @import("string.zig");
pub const stdio = @import("stdio.zig");
pub const stdlib = @import("stdlib.zig");
pub const extra = @import("extra.zig");
pub const dirent = @import("dirent.zig");
pub const private = @import("private.zig");

comptime {
    // TODO: figure out a method to not export unused stuff
    if (builtin_info.output_mode == .Lib) {
        _ = sys;
        _ = ctype;
        _ = string;
        _ = stdio;
        _ = stdlib;
        _ = extra;
        _ = dirent;
    }
    _ = private;
}

pub export fn exit(code: usize) noreturn {
    syscalls.exit(code);
    while (true) {
        asm volatile ("hlt");
    }
}

pub fn panic(msg: []const u8, error_return_trace: ?*builtin.StackTrace, return_addr: ?usize) noreturn {
    @setCold(true);
    const at = return_addr orelse @returnAddress();
    stdio.zprintf("\x1B[38;2;200;0;0mlibc panic: %.*s at %p <??>\n", .{ msg.len, msg.ptr, at }) catch {};

    stdio.zprintf("trace:\n", .{}) catch {};
    if (error_return_trace) |trace| {
        const addresses = trace.instruction_addresses;

        for (addresses) |address| {
            stdio.zprintf("  <%p>\n", .{address}) catch {};
        }
    } else {
        var rbp: ?[*]usize = @ptrFromInt(@frameAddress());
        while (rbp != null) : (rbp = @ptrFromInt(rbp.?[0])) {
            stdio.zprintf("  %p <??>\n", .{rbp.?[1]}) catch {};
        }
    }
    stdio.zprintf("\x1B[0m", .{}) catch {};

    exit(1);
}

/// sets to c main
extern fn main(argc: usize, argv: [*]const [*:0]const c_char) i32;
const root = @import("root");

fn zmain() callconv(.C) i32 {
    root.main() catch |err| return @intFromError(err);
    return 0;
}

comptime {
    if (@hasDecl(root, "main") and builtin_info.os.tag == .freestanding) {
        const info = @typeInfo(@TypeOf(root.main));
        if (info.Fn.calling_convention == .Unspecified)
            @export(zmain, .{ .name = "main" });
    }
}
/// starts as C
fn __libc_c_start() callconv(.Naked) i32 {
    // converting argv to **char
    asm volatile (
        \\ # jmps to main if argc == 0
        \\ test %rdi, %rdi
        \\ jz main
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

fn _redirect_start() callconv(.Naked) i32 {
    asm volatile (
        \\ jmp __libc_c_start
    );
}
fn _start() callconv(.Naked) noreturn {
    asm volatile (
        \\ movq $0, %rbp
        \\ push %rbp
        \\ push %rbp
        \\ push %rdi
        \\ push %rsi
        \\ call __lib__init__        
        \\ pop %rsi
        \\ pop %rdi
        \\ call _redirect_start
        \\ mov %rax, %rdi
        \\ call exit
        \\ hlt
    );
}

// we cannot export start directly to avoid problems with headergen
comptime {
    if (builtin_info.os.tag == .freestanding) {
        @export(_start, .{ .name = "_start" });

        @export(__libc_c_start, .{ .name = "__libc_c_start" });
        @export(_redirect_start, .{ .name = "_redirect_start" });
    }
}
