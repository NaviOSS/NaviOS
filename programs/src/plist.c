#include <stdio.h>
#include <utils.h>
int main(size_t argc, OsStr **argv) {
  SysInfo info = {};
  sysinfo(&info);
  ProcessInfo processes[info.processes_count];

  pcollect(processes, info.processes_count);

  printf("name:  pid  ppid\n");
  for (int i = 0; i < info.processes_count; i++) {
    ProcessInfo process = processes[i];

    printf("%s:  %d  %d\n", process.name, process.pid, process.ppid);
  }
  return 0;
}
