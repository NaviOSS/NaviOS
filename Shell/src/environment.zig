const eql = @import("utils.zig").eql;
const ArrayList = @import("utils.zig").ArrayList;

pub const EnvironmentVariable = struct {
    name: []const u8,
    value: []const u8,
};

var environment: ArrayList(EnvironmentVariable) = undefined;

pub fn init() !void {
    environment = try ArrayList(EnvironmentVariable).init();

    try environment.append(.{ .name = "PATH", .value = "sys:/bin" });
}

pub fn get_environment_variable(name: []const u8) ?[]const u8 {
    for (environment.items) |env| {
        if (eql(u8, env.name, name)) {
            return env.value;
        }
    }
    return null;
}

pub fn get_path() !ArrayList([]const u8) {
    var path = try ArrayList([]const u8).init();
    // adding current dir
    try path.append(".");

    const path_env = get_environment_variable("PATH") orelse return path;

    var current_start: usize = 0;
    for (path_env, 0..) |path_part, i| {
        if (path_part == ';') {
            try path.append(path_env[current_start..i]);
            current_start = i + 1;
        }

        if (i == path_env.len - 1) {
            try path.append(path_env[current_start..]);
        }
    }

    return path;
}
