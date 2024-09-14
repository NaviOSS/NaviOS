//! compile with `gcc -no-pie -nostdlib -static -ffreestanding -fno-stack-protector user/test.c -o user/test` if you have gcc
// you just need to compile an ELF with type EXE ig
// NOTE: i dont know C code below may be garbage but it is for test purposes
#include <stdarg.h>
#include <stdint.h>
#include <stddef.h>

static inline int64_t syscall(uint64_t num, uint64_t arg0, uint64_t arg1, uint64_t arg3, uint64_t arg4) {
    int64_t result;
    asm volatile (
        "mov %1, %%rax\n" 
        "mov %2, %%rdi\n"  
        "mov %3, %%rsi\n"  
        "mov %4, %%rdx\n"          
	"mov %5, %%rcx\n"  
        "int $0x80\n"      
        "mov %%rax, %0\n"  
        : "=r" (result)    
        : "r" (num), "r" (arg0), "r" (arg1), "r" (arg3), "r" (arg4)
        : "rax", "rdi", "rsi", "rdx"  
    );
    return result;
}

int printf(const char* format, ...);

size_t strlen(const char* str) {
	size_t len = 0;
	const char* ch = str;

	while (*ch != 0) {
		ch++;
		len++;
	}
	
	return len;
}

int64_t open(const void* path_ptr, size_t len) {
	return syscall(2, (uint64_t) path_ptr, len, 0, 0);
}

int64_t close(uint32_t fd) {
	return syscall(5, fd, 0, 0, 0);
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

/// wries the textual version of `val` into ptr
/// returns the start of the written text it will write from backwards
int itoa(int val, char ptr[10]) {
	int i = 9;
	for (; i != 0; i--) {
		int ch = val % 10;
		val /= 10;
		ptr[i] = ch + '0';

		if (val == 0) {
			break;
		}
	}

	return i;
}

void sputc(char c) {
	writeout(&c, 1);
}

int printf(const char* format, ...) {
	int i = 0;

	const char* current;
	va_list arg;
	va_start(arg, format);
	
	for (current = format; *current != '\0'; current++) {
		while (*current != '%') {
			writeout(current, 1);
			current++;
			if (*current == '\0') return 0;
		}

		current++;
		switch (*current) {
			case 'd':
				 i = va_arg(arg, int);
				if (i < 0) {
					i = -i;
					sputc('-');
				}
				
				char result[10];
				int start = itoa(i, result);
				writeout(&result[start], 10 - start);
			break;
			case 's':
				print(va_arg(arg, char*));
			break;
			case '.':
				current++;
				switch (*current) {
					case '*':						
						current++;
						int num = va_arg(arg, int);
						switch (*current) {
							case 's':
								writeout(va_arg(arg, char*), num);
							break;
						}
					break;
				}
			break;
		}
	}
	
	va_end(arg);
	return 0;
}

char getchar() {
	char c;
	readin(&c, 1);
	return c;
}

/// gets a str from stdin places it in ptr, reads until \n
/// doesn't include \n
/// returns the length of the str
/// if max is reached it will return -1 instead
int getstr(char* ptr, int max) {
	for (int i = 0; i < max; i++) {
		char c = getchar();
		if (c == '\n') return i;
		ptr[i] = c;
	}

	if (getchar() == '\n') return max;
	return -1;
}

int main() {
	char filename[7];

	print("Hello from userspace! enter 6 or less characters with the filename: ");
	int filename_len = getstr(filename, 6);
	if (filename_len == -1) {
		print("enter 6 or less characters please\n");
		return -200;
	}

	filename_len++;
	filename[filename_len - 1] = 0;

	printf("creating %s ...\n", filename);
	
	char created = create_n("ram:/", filename);

	if (created < 0) {
		print("err creating!\n");
		return created;
	}
	
	char data[10];
	print("enter 10 or less characters of data to write to it: ");
	int data_len = getstr(data, 10);
	if (data_len == -1) {
		print("enter 10 or less characters please!\n");
		return -200;
	}

	printf("writing '%.*s' to it ... \n", data_len, data);
	
	char full_path[5 + 7] = "ram:/";
	for (int i = 0; i < filename_len; i++) 
		full_path[5 + i] = filename[i];

	// FIXME: open takes full path, but create takes the dir path and the filename?
	int64_t fd = open_n(full_path);

	if (fd < 0) {
		print("failed opening the file );\n");
		return fd;
	}

	int64_t err = write(fd, data, data_len);
	if (err < 0) {
		print("failed writing to file );\n");
		return err;
	}
	
	int64_t closed = close(fd);
	if (closed < 0) return closed;

	print("done!\n");
	
	return 0;
}

void _start() {
	int64_t err = main();
	
	if (err < 0) {
		printf("failed with err: %d\n", (int) err);
	}

	pexit();
}
