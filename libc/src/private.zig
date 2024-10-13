const malloc_init = @import("stdlib.zig").__malloc__init__;
const sys = @import("sys/root.zig");
const OsStr = sys.raw.OsStr;

export var __lib__argc: usize = 0;
export var __lib__argv: ?[*]const *const OsStr = null;
export fn __lib__init__(argc: usize, argv: [*]const *const OsStr) void {
    __lib__argc = argc;
    __lib__argv = argv;
    malloc_init();
}

pub fn __lib__argv_get() [*]const *const OsStr {
    return __lib__argv.?;
}

pub fn __lib__argc_get() usize {
    return __lib__argc;
}
