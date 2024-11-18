//! raw system structs
pub fn Slice(comptime T: type) type {
    return extern struct {
        ptr: [*]const T,
        len: usize,

        pub fn from(slice: []const T) @This() {
            return .{ .ptr = slice.ptr, .len = slice.len };
        }

        pub fn to(slice: @This()) []const T {
            return slice.ptr[0..slice.len];
        }
    };
}

pub const SpawnFlags = packed struct {
    clone_resources: bool = false,
    clone_cwd: bool = false,
    _padding: u6 = 0,
};

pub const DirEntry = extern struct { kind: u8, size: usize, name_length: usize, name: [128]u8 };

pub const SpawnConfig = extern struct {
    name: Slice(u8),
    argv: [*]const Slice(u8),
    argc: usize,
    flags: SpawnFlags,
};

pub const SysInfo = extern struct { total_mem: usize, used_mem: usize, processes_count: usize };

pub const ProcessStatus = enum(u8) {
    Waiting,
    Running,
    Zombie,
};

pub const ProcessInfo = extern struct { ppid: u64, pid: u64, name: [64]u8, status: ProcessStatus, resource_count: usize, exit_code: usize, exit_addr: usize, exit_stack_addr: usize, killed_by: u64, data_start: usize, data_break: usize };

pub const OsStr = extern struct {
    len: usize,
    data_off: [1]u8,
    pub fn data(self: *@This()) [*]u8 {
        return @ptrCast(&self.data_off[self.data_off.len - 1]);
    }
};
