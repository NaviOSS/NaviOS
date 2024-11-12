const io = @import("sys/io.zig");
const string = @import("string.zig");
const stdio = @import("stdio.zig");
const stdlib = @import("stdlib.zig");
const errors = @import("sys/errno.zig");
const seterr = errors.seterr;

pub const raw = @import("sys/raw.zig");
pub const DIR = extern struct {
    current_index: usize = 0,
    ri: isize,
    dir_ri: isize,

    pub fn next(dir: *DIR) ?raw.DirEntry {
        defer dir.current_index += 1;
        return io.zdiriter_next(dir.ri);
    }

    fn dirclose(dir: *DIR) errors.Error!void {
        try io.zdiriter_close(dir.ri);
        try io.zclose(dir.dir_ri);

        stdlib.free(dir);
    }

    pub fn close(dir: *DIR) void {
        dirclose(dir) catch unreachable;
    }
};

pub export fn opendir(path: [*:0]const c_char) ?*DIR {
    const length = string.strlen(path);
    const path_u8: [*:0]const u8 = @ptrCast(path);

    return zopendir(path_u8[0..length]) catch |err| {
        seterr(err);
        return null;
    };
}

pub fn zopendir(path: []const u8) !*DIR {
    const dir_ri = try io.zopen(path);

    const ri = try io.zdiriter_open(dir_ri);

    const dir = stdlib.zmalloc(DIR).?;
    dir.ri = ri;
    dir.dir_ri = dir_ri;

    return dir;
}

// FIXME: this is very unhealthy
pub export fn readdir(dir: *DIR) ?*raw.DirEntry {
    var entry = dir.next() orelse return null;
    return &entry;
}

pub export fn telldir(dir: *DIR) c_int {
    return @intCast(dir.current_index);
}

pub export fn closedir(dir: *DIR) c_int {
    dir.dirclose() catch |err| {
        seterr(err);
        return -1;
    };
    return 0;
}
