const errors = @import("sys/errno.zig");
const Errno = errors.Errno;
const errno_t = errors.errno_t;

pub export fn strlen(cstr: [*:0]const u8) usize {
    var i: usize = 0;

    var len: usize = 0;
    while (cstr[i] != 0) : (i += 1)
        len += 1;
    return len;
}
pub export fn strerror(errnum: errno_t) [*:0]const u8 {
    if (errnum >= @intFromEnum(Errno.Last)) {
        return "UNKNOWN";
    }
    const err: Errno = @enumFromInt(errnum);
    return @tagName(err);
}

pub export fn strerrorlen_s(errnum: errno_t) usize {
    return strlen(strerror(errnum));
}
