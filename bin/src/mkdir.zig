const libc = @import("libc");
const syscalls = libc.syscalls;
const strlen = libc.string.strlen;
const printf = libc.stdio.zprintf;

export fn main(argc: usize, argv: [*]const [*:0]const c_char) i32 {
    if (argc < 2) {
        _ = printf("expected at least the name of the directory to make\n");
        return -1;
    }

    const path = argv[1];
    const len = strlen(path);

    const path_ptr: *const u8 = @ptrCast(path);
    if (syscalls.createdir(path_ptr, len) != 0) return -1;
    return 0;
}
