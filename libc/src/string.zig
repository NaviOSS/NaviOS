const errors = @import("sys/errno.zig");
const Errno = errors.Errno;

pub export fn strlen(cstr: [*:0]const u8) usize {
    var i: usize = 0;

    var len: usize = 0;
    while (cstr[i] != 0) : (i += 1)
        len += 1;
    return len;
}
pub export fn strerror(errnum: u32) [*:0]const u8 {
    if (errnum >= @intFromEnum(Errno.Last)) {
        return "UNKNOWN";
    }
    const err: Errno = @enumFromInt(errnum);
    return @tagName(err);
}

pub export fn strerrorlen_s(errnum: u32) usize {
    return strlen(strerror(errnum));
}

pub export fn memset(str: [*]void, c: c_int, n: usize) [*]void {
    const char_str: [*]u8 = @ptrCast(str);
    const char: u8 = @intCast(c);

    for (0..n) |i| {
        char_str[i] = char;
    }

    return @ptrCast(char_str);
}
