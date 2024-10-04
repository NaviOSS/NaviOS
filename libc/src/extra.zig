pub export fn reverse(buffer: [*]u8, len: usize) [*]u8 {
    var start: usize = 0;
    var end = len - 1;

    while (start < end) {
        const tmp = buffer[start];
        buffer[start] = buffer[end];
        buffer[end] = tmp;
        start += 1;
        end -= 1;
    }

    return buffer;
}

pub export fn itoa(integer: usize, buffer: [*]u8, radix: u8) c_int {
    var val = integer;
    var i: usize = 0;
    if (radix < 2 or radix > 34) return -1;

    if (val == 0) {
        buffer[i] = '0';
        buffer[i + 1] = 0;
        return 0;
    }

    while (val != 0) {
        const rem = val % radix;
        buffer[i] = @intCast(if (rem > 9) rem - 10 + 'a' else rem + '0');

        val /= radix;
        i += 1;
    }

    buffer[i] = 0;
    _ = reverse(buffer, i);
    return 0;
}
