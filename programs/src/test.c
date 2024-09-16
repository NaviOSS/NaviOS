#include "stdio.h"
#include "sys.h"

int main() {
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

  printf("creating %s ...\n", filename);

  char created = create_n("ram:/", filename);

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

  char full_path[5 + 7] = "ram:/";
  for (int i = 0; i < filename_len; i++)
    full_path[5 + i] = filename[i];

  // FIXME: open takes full path, but create takes the dir path and the
  // filename?
  int64_t fd = open_n(full_path);

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

void _start() {
  int64_t err = main();

  if (err < 0) {
    printf("failed with err: %d\n", (int)err);
  }

  pexit();
}
