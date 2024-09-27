#include "stdio.h"
#include <stdint.h>
#include <string.h>
#include <sys.h>
#include <utils.h>

int main(size_t argc, OsStr **argv) {
  int *smth = sbrk(4096 * 2) - 4;
  *smth = 0xdeadbeef;
  void *at = sbrk(0);

  printf("got %d args! break at %p after sbrk'ing 2 pages, allocated an int "
         "with value %x\n",
         argc, at, *smth);
  for (int i = 0; i < argc; i++) {
    OsStr *arg = argv[i];
    printf("arg: %.*s\n", arg->len, arg->data);
  }

  char filename[7];

  printf(
      "Hello from userspace! enter 6 or less characters with the filename: ");
  int filename_len = getstr(filename, 6);
  if (filename_len == -1) {
    printf("enter 6 or less characters please\n");
    return -200;
  }

  filename_len++;
  filename[filename_len - 1] = 0;

  char fullpath[5 + 7] = "ram:/";
  strcat(fullpath, filename);

  printf("creating %s with path %s ...\n", filename, fullpath);

  char created = create_n(fullpath);

  if (created < 0) {
    printf("err creating!\n");
    return created;
  }

  char data[10];
  printf("enter 10 or less characters of data to write to it: ");
  int data_len = getstr(data, 10);
  if (data_len == -1) {
    printf("enter 10 or less characters please!\n");
    return -200;
  }

  printf("writing '%.*s' to it ... \n", data_len, data);

  // FIXME: open takes full path, but create takes the dir path and the
  // filename?
  int64_t fd = open_n(fullpath);

  if (fd < 0) {
    printf("failed opening the file );\n");
    return fd;
  }

  int64_t err = write(fd, data, data_len);
  if (err < 0) {
    printf("failed writing to file );\n");
    return err;
  }

  int64_t closed = close(fd);
  if (closed < 0)
    return closed;

  printf("done!\n");

  return 0;
}
