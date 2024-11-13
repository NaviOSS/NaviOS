# libc
NaviOS's libc written in zig
to compile the lib simply run
```
zig build
```
the lib will be compiled in zig-out/lib/

to generate the headers run
```
zig build headergen
```

# Philosophy
i choose zig instead of rust because it is a more low-level language, which is more suitable for a libc, it will also help me learn about writing lower-level code, and to not get bored by only doing rust; i didn't choice C because, i don't like it.
## goals
- provide a basic libc for userspace, that is at least able to sustain rust std
- make it easy to write userspace programs in zig for now; this means that it can have extra zig execulsive features
- automatically generate the headers from the zig code.
## implementation
the functions should be implemented in Zig-style and then a wrapper function in C-style should be implemented on top of it, by Zig-style i mean usage of slices, errors, anytype, etc, and by C-style i mean the implementation in existing libc related standards.

there is functions implemented as struct members which takes @This() as an argument, this counts as a Zig-style implementation.
## naming
the highest level wrapper functions in Zig-style should be named after the C standard library functions prefixed with a `z`, for example `printf` is `zprintf`, and then comes the C-style final wrapper which is just `printf`.
