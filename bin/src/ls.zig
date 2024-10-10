const libc = @import("libc");
const printf = libc.stdio.zprintf;
const Errno = libc.sys.errno;
pub const panic = libc.panic;

export fn main() i32 {
    const cwd = libc.dirent.zopendir(".") orelse return -1;

    while (libc.dirent.readdir(cwd)) |ent| {
        _ = printf("%.*s\n", ent.name_length, &ent.name);
    }

    _ = libc.dirent.closedir(cwd);
    return 0;
}
