const printf = @import("libc").stdio.zprintf;

export fn main(argc: usize, argv: [*]const [*:0]const u8) i32 {
    if (argc < 2) {
        _ = printf("expected at least one argument to echo...\n");
        return -1;
    }

    for (1..argc) |i| {
        _ = printf("%s", argv[i]);
    }
    _ = printf("\n");
    return 0;
}
