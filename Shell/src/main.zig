const libc = @import("libc");
const printf = libc.stdio.zprintf;
const getline = libc.stdio.zgetline;
const Lexer = @import("Lexer.zig");
const repl = @import("repl.zig");
const Error = libc.sys.errno.Error;
const eql = @import("utils.zig").eql;

pub const panic = libc.panic;
const ArrayList = @import("utils.zig").ArrayList;
const environment = @import("environment.zig");
var ret: u64 = 0;

pub fn prompt() Error!void {
    const cwd_buffer = try libc.stdlib.zalloc(u8, 1024);
    defer libc.stdlib.free(cwd_buffer.ptr);

    const cwd_len = try libc.sys.io.zgetcwd(cwd_buffer);
    try printf("\x1B[38;2;255;0;193m%.*s\x1B[0m ", .{ cwd_len, cwd_buffer.ptr });

    if (ret != 0) {
        try printf("\x1B[38;2;255;0;0m[%l]\x1B[0m ", .{ret});
    }

    try printf("# ", .{});
}

pub fn run(line: []const u8) Error!void {
    var tokens = try ArrayList(Lexer.Token).init();
    defer tokens.deinit();

    var lexer = Lexer.init(line);
    while (lexer.next()) |token| {
        try tokens.append(token);
    }
    if (tokens.items.len < 1) return;

    const name = tokens.items[0].asString();
    ret = repl.repl(tokens.items) catch |err| blk: {
        const err_name = @errorName(err);
        try printf("failed to execute %.*s, error: %.*s\n", .{ name.len, name.ptr, err_name.len, err_name.ptr });
        break :blk 0;
    };
}

pub fn main() Error!void {
    try printf("\x1B[38;2;255;192;203m", .{});
    try printf(
        \\  ,---.             ,---.           ,-----.   ,---.   
        \\ '   .-'   ,--,--. /  .-'  ,--,--. '  .-.  ' '   .-'  
        \\ `.  `-.  ' ,-.  | |  `-, ' ,-.  | |  | |  | `.  `-.  
        \\ .-'    | \ '-'  | |  .-' \ '-'  | '  '-'  ' .-'    | 
        \\ `-----'   `--`--' `--'    `--`--'  `-----'  `-----'  
    , .{});

    try printf("\n\x1B[38;2;200;200;200m", .{});
    try printf(
        \\| Welcome to SafaOS!
        \\| you are currently in ram:/, a playground
        \\| init ramdisk has been mounted at sys:/
        \\| sys:/bin is avalible in your PATH check it out for some binaries
        \\| the command `help` will provide a list of builtin commands and some terminal usage guide
    , .{});

    try printf("\x1B[0m\n", .{});

    try environment.init();

    while (true) {
        try prompt();
        const line = try getline();
        defer libc.stdlib.free(line.ptr);

        try run(line);
    }
}

comptime {
    _ = libc;
}
