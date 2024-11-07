pub const Error = error{
    // use when no ErrorStatus is avalible for xyz and you cannot add a new one
    Generic,
    OperationNotSupported,
    // for example an elf class is not supported, there is a difference between NotSupported and
    // OperationNotSupported
    NotSupported,
    // for example a magic value is invaild
    Corrupted,
    InvaildSyscall,
    InvaildResource,
    InvaildPid,
    // instead of panicking syscalls will return this on null and unaligned pointers
    InvaildPtr,
    // for operations that requires a vaild utf8 str...
    InvaildStr,
    InvaildPath,
    InvaildDrive,
    NoSuchAFileOrDirectory,
    NotAFile,
    NotADirectory,
    AlreadyExists,
    NotExecutable,
    // would be useful when i add remove related operations to the vfs
    DirectoryNotEmpty,
    // Generic premissions(protection) related error
    MissingPermissions,
    // memory allocations and mapping error, most likely that memory is full
    OutOfMemory,
    Busy,
    NotEnoughArguments,
    // iso
    ArgumentOutOfDomain,
    IllegalByteSequence,
    ResultOutOfRange,
    // method to identify the enum max
    Last,
};

pub export var errno: u32 = 0;
pub fn geterr() Error {
    const err: u16 = @truncate(errno);
    return @errorCast(@errorFromInt(err));
}

pub fn seterr(errnum: Error) void {
    errno = @intFromError(errnum);
}
