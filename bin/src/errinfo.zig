const libc = @import("libc");
const printf = libc.stdio.zprintf;
const panic = libc.panic;

fn parse(str: []const u8) !usize {
    var result: usize = 0;
    var power: usize = 1;
    var i: usize = str.len;

    while (i != 0) {
        i -= 1;

        if (str[i] < '0' or str[i] > '9') return error.InvalidNumber;
        const digit = str[i] - '0';

        const value = digit * power;

        power *= 10;
        result += value;
    }

    return result;
}

pub fn main() libc.sys.errno.Error!void {
    var args = libc.sys.args();

    if (args.count() < 2) return error.NotEnoughArguments;
    const errstr = args.nth(1).?;
    const errnum = parse(errstr) catch return error.ArgumentOutOfDomain;
    const errnum_tru: u16 = @truncate(errnum);

    const name = libc.string.strerror(errnum_tru);

    try printf("%s\n", .{name});
}

comptime {
    _ = libc;
}
