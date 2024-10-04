//! generates header from a zig library
//! this was designed for this project ONLY
//! as always the code here is really bad, rushed and designed for my usecases ONLY
// settings:
/// the directory where the headers should be generated relative from cwd
const outdir = "includes";
/// the dierctory of the library src, relative to cwd
const srcdir = "src";
/// library name
const libname = "nlibc";
/// an import to the lib root file
/// the lib root must contain `pub const`s of `@import`s to all the sub headers which should be generated
const libroot = @import("src/root.zig");
/// headers to be included in every header
const default_includes = [_][]const u8{
    "stddef.h",
    "stdint.h",
    "stdbool.h",
    "sys/types.h",
};

pub fn main() !void {
    std.debug.print("generating headers ... \n", .{});
    std.fs.cwd().deleteDir(outdir) catch {};
    var arena = std.heap.ArenaAllocator.init(std.heap.page_allocator);
    defer arena.deinit();

    var creator = try Creator.init(arena.allocator(), srcdir);
    try creator.create_headers_from_root(libroot);

    try creator.finish();
}

const std = @import("std");
const ContainerLayout = std.builtin.Type.ContainerLayout;

pub fn type_sname(comptime ty: type) []const u8 {
    comptime var start = 0;
    const name = @typeName(ty);

    inline for (name, 0..) |c, i| {
        if (c == '.') start = i + 1;
    }

    return name[start..];
}

/// list directory contents recursively and return an array of paths
/// NOTE: base_path should be equal to current_path on first call
fn list_directory_recursive(base_path: []const u8, current_path: []const u8, allocator: std.mem.Allocator) ![][]u8 {
    var paths = std.ArrayList([]u8).init(allocator);

    var dir = try std.fs.cwd().openDir(current_path, std.fs.Dir.OpenDirOptions{ .access_sub_paths = true, .iterate = true });
    defer dir.close();

    var it = dir.iterate();

    while (try it.next()) |entry| {
        const full_path = std.fs.path.join(allocator, &.{ current_path, entry.name }) catch return error.OutOfMemory;

        if (std.mem.eql(u8, entry.name, ".") or std.mem.eql(u8, entry.name, "..")) {
            continue;
        }

        const relative_path = full_path[base_path.len + 1 ..]; // Skip base directory and '/'
        try paths.append(relative_path);

        if (entry.kind == .directory) {
            const sub_paths = try list_directory_recursive(base_path, full_path, allocator);

            for (sub_paths) |sub_path| {
                try paths.append(sub_path);
            }
        }
    }

    return paths.toOwnedSlice();
}

pub const Generator = struct {
    buffer: std.ArrayList(u8),
    vaild_structs: std.ArrayList([]const u8),
    external_structs: std.ArrayList([]const u8),
    ident: usize = 0,

    allocator: std.mem.Allocator,
    empty: bool = true,

    pub fn append_structs(self: *@This(), structs: [][]const u8) !void {
        for (structs) |item| {
            try self.external_structs.append(item);
        }
    }

    fn init(allocator: std.mem.Allocator) !@This() {
        const external_structs = std.ArrayList([]const u8).init(allocator);

        return .{ .buffer = std.ArrayList(u8).init(allocator), .vaild_structs = std.ArrayList([]const u8).init(allocator), .allocator = allocator, .external_structs = external_structs };
    }

    fn is_vaild(self: *@This(), name: []const u8) bool {
        for (self.vaild_structs.items) |item| {
            if (std.mem.eql(u8, item, name))
                return true;
        }

        for (self.external_structs.items) |item| {
            if (std.mem.eql(u8, item, name))
                return true;
        }
        return false;
    }

    fn appendident(self: *@This()) !void {
        for (0..self.ident) |_| {
            try self.buffer.append(' ');
        }
    }

    fn append(self: *@This(), slice: []const u8) !void {
        try self.buffer.appendSlice(slice);
    }

    fn appendln(self: *@This(), slice: []const u8) !void {
        try self.appendident();
        try self.buffer.appendSlice(slice);
        try self.buffer.append('\n');
        self.ident += 2;
    }

    fn appendf(self: *@This(), comptime fmt: []const u8, args: anytype) !void {
        const writer = self.buffer.writer();

        try std.fmt.format(writer, fmt, args);
    }

    fn appendfs(self: *@This(), comptime fmt: []const u8, args: anytype) !void {
        try self.appendident();
        const writer = self.buffer.writer();

        try std.fmt.format(writer, fmt, args);
    }

    fn appendfln(self: *@This(), comptime fmt: []const u8, args: anytype) !void {
        try self.appendident();
        const writer = self.buffer.writer();

        try std.fmt.format(writer, fmt, args);
        try self.buffer.append('\n');
        self.ident += 2;
    }

    fn finish(self: *@This()) void {
        if (self.ident >= 2)
            self.ident -= 2;
    }

    fn append_type(self: *@This(), ty: type, name: []const u8, append_ident: bool) !void {
        const raw = blk: {
            break :blk switch (ty) {
                u8 => "uint8_t",
                u16 => "uint16_t",
                u32 => "uint32_t",
                u64 => "uint64_t",
                usize => "size_t",
                isize => "ssize_t",
                i64 => "int64_t",
                i32 => "int32_t",
                i16 => "int16_t",
                i8 => "int8_t",
                c_char => "char",
                c_int => "int",
                c_uint => "unsigned int",
                bool => "bool",
                void => "void",
                inline else => |other| {
                    const info = @typeInfo(other);
                    switch (info) {
                        .Struct => |str| {
                            switch (str.layout) {
                                .@"extern" => {
                                    const sname = type_sname(other);
                                    if (self.is_vaild(sname))
                                        break :blk sname
                                    else {
                                        if (append_ident) try self.appendident();
                                        try self.generate_struct(other, true);
                                        try self.appendf(" {s}", .{name});
                                        return;
                                    }
                                },
                                .@"packed" => if (@sizeOf(other) == 1) break :blk "uint8_t" else return error.PackedNotImplemented,
                                else => return error.NotExtern,
                            }
                        },

                        .Pointer => |pointer| {
                            if (pointer.is_const) {
                                try self.appendfs("const ", .{});
                            }

                            self.append_type(pointer.child, "", false) catch try self.append("void ");
                            return self.appendf("*{s}", .{name});
                        },

                        .Optional => |optional| {
                            return self.append_type(optional.child, name, append_ident);
                        },

                        .Array => |array| {
                            try self.append_type(array.child, "", true);
                            return self.appendf("{s}[{}]", .{ name, array.len });
                        },
                        .Enum => break :blk type_sname(other),
                        else => return error.UnsupportedType,
                    }
                },
            };
        };

        if (append_ident)
            try self.appendfs("{s} {s}", .{ raw, name })
        else
            try self.appendf("{s} {s}", .{ raw, name });
    }

    pub fn generate_struct(self: *@This(), s: type, anonymous: bool) !void {
        const info = @typeInfo(s).Struct;
        if (info.layout != ContainerLayout.@"extern") {
            return;
        }
        const name = type_sname(s);

        if (!anonymous)
            try self.appendfln("typedef struct {s} {{", .{name})
        else {
            try self.append("struct {\n");
            self.ident += 2;
        }

        inline for (info.fields) |field| {
            try self.append_type(field.type, field.name, true);
            try self.append(";\n");
        }

        self.finish();

        if (!anonymous)
            try self.appendfs("}} {s};\n\n", .{name})
        else
            try self.appendfs("}}", .{});

        if (!anonymous)
            try self.vaild_structs.append(name);

        self.empty = false;
    }

    pub fn generate_enum(self: *@This(), s: type) !void {
        const info = @typeInfo(s).Enum;
        const name = type_sname(s);
        try self.appendf("typedef enum {s}: ", .{name});
        try self.append_type(info.tag_type, "", false);
        try self.appendln("{");

        inline for (info.fields) |field| {
            try self.appendfs("{s}, \n", .{field.name});
        }

        self.finish();
        try self.appendfs("}} {s};\n\n", .{name});

        try self.vaild_structs.append(name);

        self.empty = false;
    }

    pub fn generate_function(self: *@This(), f: type, name: []const u8) !void {
        const info = @typeInfo(f).Fn;

        if (info.calling_convention != .C) {
            return;
        }

        std.debug.print("generating function {s} ... \n", .{name});
        try self.append_type(info.return_type.?, name, true);
        try self.append("(");

        inline for (info.params, 0..) |param, i| {
            try self.append_type(param.type.?, "", false);
            try self.appendf("arg{}", .{i});

            if (i + 1 < info.params.len)
                try self.append(", ");
        }

        if (info.is_var_args) try self.append(", ...");
        try self.append(");\n");

        self.empty = false;
    }
    pub fn generate_type(self: *@This(), ty: type) !void {
        switch (@typeInfo(ty)) {
            .Struct => try self.generate_struct(ty, false),
            .Enum => try self.generate_enum(ty),
            else => return error.UnGeneratableType,
        }
    }

    /// generates an #include "{path}"
    /// path can either be PARENT/CHILD or PARENT.CHILD we will handle it anyways
    pub fn generate_include(self: *@This(), path: []const u8) !void {
        var path_copy = try self.allocator.dupe(u8, path);

        for (path_copy, 0..) |c, i| {
            if (c == '.') {
                path_copy[i] = '/';
            }
        }

        try self.appendfs("#include \"{s}.h\"\n", .{path_copy});
        self.allocator.free(path_copy);
    }

    pub fn generate_var(self: *@This(), name: []const u8, ty: type) !void {
        try self.append("extern ");
        try self.append_type(ty, name, false);
        try self.append(";\n");

        self.empty = false;
    }

    pub fn generate_needed(self: *@This(), type_name: []const u8) !void {
        const dup = try self.allocator.dupe(u8, type_name);

        for (dup, 0..) |c, i| {
            if (c == '.') {
                dup[i] = '_';
                continue;
            }

            dup[i] = std.ascii.toUpper(c);
        }

        try self.appendf("#ifndef __{s}__{s}_\n#define __{0s}__{1s}_\n\n", .{ libname, dup });

        for (default_includes) |header| {
            try self.appendf("#include <{s}>\n", .{header});
        }
        try self.append("\n");

        self.allocator.free(dup);
    }

    pub fn deinit(self: *@This()) void {
        self.external_structs.deinit();
        self.vaild_structs.deinit();
        self.buffer.deinit();
    }
    /// deinits and finishes generation returning the generated header
    pub fn finish_mod(self: *@This()) ![]const u8 {
        defer self.deinit();

        try self.append("\n#endif");
        return self.buffer.toOwnedSlice();
    }
};

pub const Creator = struct {
    generated: std.StringHashMap([][]const u8),
    making: []const []const u8,
    allocator: std.mem.Allocator,

    /// converts a header path to a type name
    fn header_to_type(self: *@This(), header: []const u8) ![]const u8 {
        const dup = try self.allocator.dupe(u8, header);

        for (dup, 0..) |c, i| {
            if (c == '/') {
                dup[i] = '.';
            }
        }
        return dup[0 .. header.len - 2];
    }
    /// returns the actual `path` start where it's relative to `mod_path`
    fn header_path_relative(path: []const u8, mod_path: []const u8) usize {
        var last_slash: usize = 0;
        for (path, 0..) |c, i| {
            if (c == '/') last_slash = i + 1;
            if (c != mod_path[i]) break;
        }
        return last_slash;
    }
    /// gets a header from type name
    fn get_header_path(self: *@This(), name: []const u8) !?[]const u8 {
        var name_search = name;
        if (std.mem.startsWith(u8, name, srcdir ++ ".")) {
            name_search = name[srcdir.len + 1 ..];
        }

        for (self.making) |header| {
            const ty = try self.header_to_type(header);
            if (std.mem.eql(u8, ty, name_search)) {
                return header;
            }

            self.allocator.free(ty);
        }

        return null;
    }

    fn get_header_path_relative(self: *@This(), name: []const u8, mod_name: []const u8) !?[]const u8 {
        const path = try self.get_header_path(name) orelse return null;
        const mod_path = try self.get_header_path(mod_name) orelse return null;

        const path_start = header_path_relative(path, mod_path);
        return path[path_start..];
    }

    fn is_vaild_header(self: *@This(), name: []const u8) bool {
        var iterator = self.generated.keyIterator();
        while (iterator.next()) |key| {
            if (std.mem.eql(u8, key.*, name)) return true;
        }

        return false;
    }

    pub fn init(allocator: std.mem.Allocator, directory: []const u8) !@This() {
        const making = try list_directory_recursive(directory, directory, allocator);

        std.fs.cwd().makeDir(outdir) catch |err|
            if (err != error.PathAlreadyExists) return err;

        var src = try std.fs.cwd().openDir(directory, .{});
        var includes = try std.fs.cwd().openDir(outdir, .{});

        for (making, 0..) |make, i| {
            if (!std.mem.containsAtLeast(u8, make, 1, &[_]u8{'.'})) {
                includes.makeDir(make) catch |err|
                    if (err != error.PathAlreadyExists) return err;
                continue;
            }

            // if a header copy it to includes
            if (std.mem.endsWith(u8, make, ".h")) {
                const file = try includes.createFile(make, .{});
                const src_file = try src.openFile(make, .{ .mode = .read_only });

                const buffer = try src_file.readToEndAlloc(allocator, std.math.maxInt(usize));
                try file.writeAll(buffer);

                allocator.free(buffer);
                src_file.close();
                file.close();
                continue;
            }

            if (std.mem.endsWith(u8, make, ".zig")) {
                const idx = std.mem.indexOf(u8, make, ".zig").?;
                make[idx + 1] = 'h';
                making[i] = make[0 .. idx + 2];

                const file = try includes.createFile(making[i], .{});
                file.close();
            }
        }

        includes.close();
        src.close();
        return .{ .generated = std.StringHashMap([][]const u8).init(allocator), .allocator = allocator, .making = making };
    }

    pub fn create_mod(self: *@This(), comptime ty: type) ![]const u8 {
        var generator = try Generator.init(self.allocator);

        const info = @typeInfo(ty).Struct;
        const name = @typeName(ty);

        std.debug.print("generating {s} ... \n", .{name});
        try generator.generate_needed(name);

        inline for (info.decls) |decl| {
            const field = @field(ty, decl.name);

            const field_ty = @TypeOf(field);
            const field_info = @typeInfo(field_ty);

            switch (field_info) {
                .Fn => {
                    try generator.generate_function(@TypeOf(field), decl.name);
                },
                .Type => {
                    const child_info = @typeInfo(field);
                    switch (child_info) {
                        .Struct => |s| {
                            const field_name = @typeName(field);

                            if (s.layout == ContainerLayout.auto) {
                                if (self.is_vaild_header(field_name))
                                    try generator.generate_include(field_name)
                                else if (try self.get_header_path_relative(field_name, name)) |path| {
                                    const path_abs = try self.get_header_path(field_name);

                                    try self.create_mod_to(field, path_abs.?);
                                    try generator.generate_include(path[0 .. path.len - 2]);
                                    if (self.generated.get(field_name)) |structs| {
                                        try generator.append_structs(structs);
                                    }
                                }

                                continue;
                            }
                        },
                        else => {},
                    }

                    try generator.generate_type(field);
                },
                inline else => {
                    var err: bool = false;
                    generator.generate_var(decl.name, field_ty) catch {
                        err = true;
                    };
                },
            }
        }

        if (generator.empty) {
            generator.deinit();
            return &[_]u8{};
        }

        try self.generated.put(name, try generator.vaild_structs.toOwnedSlice());

        return generator.finish_mod();
    }

    /// creates a header and puts it in `path` relative from `outdir`
    fn create_mod_to(self: *@This(), comptime ty: type, path: []const u8) (std.fs.File.OpenError || std.fs.File.WriteError)!void {
        const dir = try std.fs.cwd().openDir(outdir, .{});
        const file = try dir.openFile(path, .{ .mode = .write_only });
        const data = self.create_mod(ty) catch |err| {
            std.debug.print("failed creating header from ty " ++ @typeName(ty) ++ " error: {s}\n", .{@errorName(err)});
            @panic("failed creating header");
        };

        _ = try file.write(data);
    }

    /// creates all the headers from a given root
    pub fn create_headers_from_root(self: *@This(), comptime root: type) !void {
        const path = (try self.get_header_path(@typeName(root))).?;
        try self.create_mod_to(root, path);
    }

    pub fn deinit(self: *@This()) void {
        self.generated.deinit();
    }

    pub fn finish(self: *@This()) !void {
        self.deinit();
        var dir = try std.fs.cwd().openDir(outdir, .{ .iterate = true });

        // deleting empty files
        for (self.making) |file_path| {
            if (std.mem.endsWith(u8, file_path, ".h")) {
                const file = try dir.openFile(file_path, .{ .mode = .read_only });
                const stat = try file.stat();

                file.close();
                if (stat.size == 0) try dir.deleteFile(file_path);
            }
        }

        // deleting empty directories
        var it = dir.iterate();
        while (try it.next()) |entry| {
            if (entry.kind == .directory) {
                var sub_dir = try dir.openDir(entry.name, .{ .iterate = true });
                var sub_it = sub_dir.iterate();

                if (try sub_it.next() == null) {
                    try dir.deleteDir(entry.name);
                }
                sub_dir.close();
            }
        }
        dir.close();
    }
};
