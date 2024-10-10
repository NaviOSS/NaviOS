const libc = @import("libc");
const printf = libc.stdio.zprintf;
const Errno = libc.sys.errno;

export fn main() i32 {
    const cwd = libc.dirent.zopendir(".") orelse return -1;

    while (libc.dirent.readdir(cwd)) |ent| {
        if (ent.kind == 1)
            _ = printf("\x1B[38;2;20;255;0m%.*s\n\x1B[0m", ent.name_length, &ent.name)
        else
            _ = printf("%.*s\n", ent.name_length, &ent.name);
    }

    _ = libc.dirent.closedir(cwd);
    return 0;
}
