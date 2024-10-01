#ifndef __nlibc__SRC_SYS_RAW_
#define __nlibc__SRC_SYS_RAW_

#include <stddef.h>
#include <stdint.h>
#include <unistd.h>
#include <stdbool.h>

typedef struct DirEntry {
  uint8_t kind;
  size_t size;
  size_t name_length;
  uint8_t name[128];
} DirEntry;

typedef struct SpawnConfig {
  struct {
    const uint8_t *ptr;
    size_t len;
  } name;
  const struct {
    const uint8_t *ptr;
    size_t len;
  } *argv;
  size_t argc;
  uint8_t flags;
} SpawnConfig;

typedef struct SysInfo {
  size_t total_mem;
  size_t used_mem;
  size_t processes_count;
} SysInfo;

typedef enum ProcessStatus: uint8_t {
  Waiting, 
  Running, 
  WaitingForBurying, 
} ProcessStatus;

typedef struct ProcessInfo {
  uint64_t ppid;
  uint64_t pid;
  uint8_t name[64];
  ProcessStatus status;
} ProcessInfo;


#endif