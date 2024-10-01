//! raw system structs
pub fn Slice(comptime T: type) type {
    return struct {
        ptr: *const T,
        len: usize,
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
    argv: *const Slice(u8),
    argc: usize,
    flags: SpawnFlags,
};

pub const SysInfo = extern struct { total_mem: usize, used_mem: usize, processes_count: usize };

pub const ProcessStatus = enum(u8) {
    Waiting,
    Running,
    WaitingForBurying,
};

pub const ProcessInfo = extern struct { ppid: u64, pid: u64, name: [64]u8, status: ProcessStatus };
