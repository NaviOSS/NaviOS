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
            const items = try libc.stdlib.zalloc(T, 0);
            return Self{ .items = items, .capacity = 0 };
        }

        pub fn deinit(self: *Self) void {
            libc.stdlib.free(@ptrCast(self.items.ptr));
        }

        pub fn extend_capacity_by(self: *Self, amount: usize) !void {
            self.capacity += amount;
            const realloc = libc.stdlib.realloc(@ptrCast(self.items.ptr), @sizeOf(T) * self.capacity) orelse return error.OutOfMemory;
            self.items.ptr = @ptrCast(@alignCast(realloc));
        }

        pub fn set_len(self: *Self, len: usize) !void {
            if (len > self.capacity) {
                try self.extend_capacity_by(len - self.capacity);
            }
            self.items.len = len;
        }

        pub fn append(self: *Self, item: T) !void {
            if (self.items.len == self.capacity) {
                try self.extend_capacity_by(1);
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
