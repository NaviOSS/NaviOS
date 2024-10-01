#ifndef __nlibc__SRC_SYS_ERRNO_
#define __nlibc__SRC_SYS_ERRNO_

#include <stddef.h>
#include <stdint.h>
#include <unistd.h>
#include <stdbool.h>

typedef enum Errno: uint32_t {
  None, 
  Generic, 
  OperationNotSupported, 
  NotSupported, 
  Corrupted, 
  InvaildSyscall, 
  InvaildResource, 
  InvaildPid, 
  InvaildPtr, 
  InvaildStr, 
  InvaildPath, 
  InvaildDrive, 
  NoSuchAFileOrDirectory, 
  NotAFile, 
  NotADirectory, 
  AlreadyExists, 
  NotExecutable, 
  DirectoryNotEmpty, 
  MissingPermissions, 
  MMapError, 
  ArgumentOutOfDomain, 
  IllegalByteSequence, 
  ResultOutOfRange, 
  Last, 
} Errno;


#endif