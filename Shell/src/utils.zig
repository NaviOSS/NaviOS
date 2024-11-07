pub fn eql(comptime T: type, a: []const T, b: []const T) bool {
    if (a.len != b.len) return false;
    for (a, 0..) |item, i| {
        if (item != b[i]) return false;
    }
    return true;
}

const libc = @import("libc");

pub fn ArrayList(comptime T: type) type {
    return struct {
        const Self = @This();
        const Slice = []T;
        items: Slice,
        capacity: usize,

        pub fn init() !Self {
            const items = libc.stdlib.zalloc(T, 0) orelse return error.OutOfMemory;
            return Self{ .items = items, .capacity = 0 };
        }

        pub fn deinit(self: *Self) void {
            libc.stdlib.free(self.items.ptr);
        }

        fn extend_capacity(self: *Self) !void {
            self.capacity += 1;
            const realloc = libc.stdlib.realloc(@ptrCast(self.items.ptr), @sizeOf(T) * self.capacity) orelse return error.OutOfMemory;
            self.items.ptr = @ptrCast(@alignCast(realloc));
        }

        pub fn append(self: *Self, item: T) !void {
            if (self.items.len == self.capacity) {
                try self.extend_capacity();
            }
            self.items.len += 1;
            self.items[self.items.len - 1] = item;
        }

        pub fn append_slice(self: *Self, slice: []const T) !void {
            for (slice) |item| {
                try self.append(item);
            }
        }
    };
}
