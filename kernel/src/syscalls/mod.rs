// TODO: make a proc-macro that generates the syscalls from rust functions
// for example it should generate a pointer and a length from a slice argument checking if it is vaild and
// returning invaild ptr if it is not
// it should also support optional pointer-arguments using Option<T>
// and we should do something about functions that takes a struct
mod io;
mod processes;
mod utils;
