const io = @import("sys/io.zig");
const string = @import("string.zig");
const stdio = @import("stdio.zig");
const stdlib = @import("stdlib.zig");

pub const raw = @import("sys/raw.zig");
pub const DIR = extern struct {
    current_index: usize = 0,
    ri: isize,
    dir_ri: isize,
};

pub export fn opendir(path: [*:0]const c_char) ?*DIR {
    const dir_ri = io.open(@ptrCast(path), string.strlen(path));
    if (dir_ri < 0) return null;

    const ri = io.diriter_open(dir_ri);
    if (ri < 0) return null;

    const dir = stdlib.zmalloc(DIR).?;
    dir.ri = ri;
    dir.dir_ri = dir_ri;

    return dir;
}

pub fn zopendir(path: [*:0]const u8) ?*DIR {
    return opendir(@ptrCast(path));
}

pub export fn readdir(dir: *DIR) ?*raw.DirEntry {
    defer dir.current_index += 1;
    return io.diriter_next(dir.ri);
}

pub export fn telldir(dir: *DIR) c_int {
    return @intCast(dir.current_index);
}

pub export fn closedir(dir: *DIR) c_int {
    const err = io.diriter_close(dir.ri);
    if (err < 0) return -1;

    const err1 = io.close(dir.dir_ri);
    if (err1 < 0) return -1;

    stdlib.free(dir);
    return 0;
}
