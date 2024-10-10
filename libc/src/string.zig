const errors = @import("sys/errno.zig");
const Errno = errors.Errno;

pub export fn strlen(cstr: [*:0]const c_char) usize {
    var i: usize = 0;
    while (cstr[i] != 0)
        i += 1;
    return i;
}
pub export fn strerror(errnum: u32) [*:0]const c_char {
    if (errnum >= @intFromEnum(Errno.Last)) {
        return @ptrCast("UNKNOWN");
    }
    const err: Errno = @enumFromInt(errnum);
    return @ptrCast(@tagName(err));
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

pub export fn memcpy(dest: [*]void, src: [*]const void, size: usize) [*]void {
    const byte_dest: [*]u8 = @ptrCast(dest);
    const byte_src: [*]const u8 = @ptrCast(src);
    for (0..size) |i| {
        byte_dest[i] = byte_src[i];
    }

    return dest;
}
