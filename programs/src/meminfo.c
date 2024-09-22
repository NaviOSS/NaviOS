#include <stdio.h>
#include <utils.h>
int main(size_t argc, Str **argv) {
  SysInfo info = {};
  sysinfo(&info);
  size_t mem_ava = info.mem_total - info.mem_used;

  printf("memory info:\n");
  printf("%dB used of %dB, %dB usable\n", info.mem_used, info.mem_total,
         mem_ava);

  printf("%dKiBs used of %dKiBs, %dKiBs usable\n", info.mem_used / 1024,
         info.mem_total / 1024, mem_ava / 1024);

  printf("%dMiBs used of %dMiBs, %dMiBs usable\n", info.mem_used / 1024 / 1024,
         info.mem_total / 1024 / 1024, mem_ava / 1024 / 1024);
  return 0;
}
