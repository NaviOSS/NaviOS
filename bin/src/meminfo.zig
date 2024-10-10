const libc = @import("libc");
const sysinfo = libc.sys.utils.sysinfo;
const printf = libc.stdio.zprintf;
export fn main() i32 {
    const info = sysinfo().?.*;
    const mem_ava: usize = info.total_mem - info.used_mem;

    _ = printf("memory info:\n");
    _ = printf("%dB used of %dB, %dB usable\n", info.used_mem, info.total_mem, mem_ava);

    _ = printf("%dKiBs used of %dKiBs, %dKiBs usable\n", info.used_mem / 1024, info.total_mem / 1024, mem_ava / 1024);

    _ = printf("%dMiBs used of %dMiBs, %dMiBs usable\n", info.used_mem / 1024 / 1024, info.total_mem / 1024 / 1024, mem_ava / 1024 / 1024);
    return 0;
}
