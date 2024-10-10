const libc = @import("libc");
const printf = libc.stdio.zprintf;

export fn main(argc: usize, argv: [*]const [*:0]const u8) i32 {
    if (argc < 2) {
        _ = printf("expected filename to cat\n");
        return -1;
    }

    const filename = argv[1];
    const file = libc.stdio.zfopen(filename, "") orelse {
        _ = printf("cat couldn't open file '%s'!\n", filename);
        return -1;
    };

    var len: usize = 0;
    const data = libc.stdio.fgetline(file, &len);
    _ = printf("%.*s\n", len, data);

    _ = libc.stdio.fclose(file);
    return 0;
}
