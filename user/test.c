//! compile with `gcc -no-pie -nostdlib -static -ffreestanding -fno-stack-protector user/test.c -o user/test` if you have gcc
// you just need to compile an ELF with type EXE ig
// NOTE: i dont know C code below may be garbage but it is for test purposes
#include <stdint.h>
#include <stddef.h>

static inline int64_t syscall(uint64_t num, uint64_t arg0, uint64_t arg1, uint64_t arg3, uint64_t arg4) {
    int64_t result;
    asm volatile (
        "mov %1, %%rax\n" 
        "mov %2, %%rdi\n"  
        "mov %3, %%rsi\n"  
        "mov %4, %%rdx\n"          
	"mov %5, %%r8\n"  
        "int $0x80\n"      
        "mov %%rax, %0\n"  
        : "=r" (result)    
        : "r" (num), "r" (arg0), "r" (arg1), "r" (arg3), "r" (arg4)
        : "rax", "rdi", "rsi", "rdx"  
    );
    return result;
}

size_t strlen(const char* str) {
	size_t len = 0;
	const char* ch = str;

	while (*ch != 0) {
		ch += 1;
		len += 1;
	}

	return len;
}

int64_t open(const void* path_ptr, size_t len) {
	return syscall(2, (uint64_t) path_ptr, len, 0, 0);
}

/// wrapper around open that takes a null-terminated path
int64_t open_n(const char* path) {
	size_t len = strlen(path);

	return open(path, len);
}

int64_t write(uint32_t fd, const void* ptr, size_t len) {
	return syscall(3, fd, (uint64_t) ptr, len, 0);
}

int64_t read(uint32_t fd, void* ptr, size_t len) {
	return syscall(4, fd, (uint64_t) ptr, len, 0);
}


int64_t create(const void* path, size_t path_len, const void* filename, size_t filename_len) {
	return syscall(6, (uint64_t) path, path_len ,(uint64_t) filename, filename_len);
}
/// wrapper around create that takes a null terminated path and a null terminated name
int64_t create_n(const char* path, const char* filename) {
	size_t path_len, filename_len;

	path_len = strlen(path);
	filename_len = strlen(filename);

	return create(path, path_len, filename, filename_len);
}

void writeout(const char* ptr, size_t len) {
	write(1, ptr, len);
}

void readin(char* ptr, size_t len) {
	read(0, ptr, len);
}

void print(const char* str) {
	size_t len = strlen(str);
	writeout(str, len);
}

void pexit() {
	syscall(0, 0, 0 , 0, 0);
}

int main() {
	char filename[7];
	filename[6] = 0;

	print("Hello from userspace! enter 6 characters with the filename: ");
	readin(filename, 6);
	
	print("\ncreating ");
	print(filename);
	print("...\n");

	if (create_n("ram:/", filename) >= 0) {
		print("success!\n");
	} else {
		print("failed );\n");
	}
	
	char data[10];
	print("enter 10 characters of data to write to it: ");
	readin(data, 10);
	
	print("\nwriting '");
	writeout(data, 10);
	print("' to it ...\n");
	
	char full_path[5 + 7] = {'r', 'a', 'm', ':', '/'};

	for (int i = 0; i < strlen(filename); i++) {
		full_path[5 + i] = filename[i];
	}

	full_path[5 + 6] = 0;
	
	// FIXME: open takes full path, but create takes the dir path and the filename?
	int64_t fd = open_n(full_path);

	if (fd < 0) {
		print("failed opening the file );\n");
		return fd;
	}

	int64_t err = write(fd, data, 10);
	if (err < 0) {
		print("failed writing to file );\n");
		return err;
	}

	print("done!\n");

	return 0;
}

void _start() {
	int64_t err = main();

	char errc = (-err) + '0';
	print("failed with err: -");
	writeout(&errc, 1);
	errc = '\n';
	writeout(&errc, 1);

	pexit();
}
