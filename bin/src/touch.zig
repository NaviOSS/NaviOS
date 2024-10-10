const libc = @import("libc");
const printf = libc.stdio.zprintf;

export fn main(argc: usize, argv: [*]const [*:0]const u8) i32 {
    if (argc < 2) {
        _ = printf("expected at least the file name to touch!\n");
        return -1;
    }

    const filename = argv[1];
    if (libc.stdio.zfopen(filename, "") != null) {
        return 0;
    }

    _ = libc.stdio.zfopen(filename, "w");
    return 0;
}
