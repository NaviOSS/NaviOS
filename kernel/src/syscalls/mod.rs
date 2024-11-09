// TODO: make a proc-macro that generates the syscalls from rust functions
// for example it should generate a pointer and a length from a slice argument checking if it is vaild and
// returning invaild ptr if it is not
// it should also support optional pointer-arguments using Option<T>
// and we should do something about functions that takes a struct
mod io;
mod processes;
mod utils;

#[macro_export]
/// makes a slice from a ptr and len
/// returns ErrorStatus::InvaildPtr if invaild
macro_rules! make_slice {
    ($ptr: expr, $len: expr) => {
        if !($ptr.is_null() && $len == 0) {
            if $ptr.is_null() || !$ptr.is_aligned() {
                return ErrorStatus::InvaildPtr;
            }

            unsafe { core::slice::from_raw_parts($ptr, $len) }
        } else {
            &[]
        }
    };
}
#[macro_export]
/// makes a mutable slice from a ptr and len
/// returns ErrorStatus::InvaildPtr if invaild
macro_rules! make_slice_mut {
    ($ptr: expr, $len: expr) => {
        if !($ptr.is_null() && $len == 0) {
            if $ptr.is_null() || !$ptr.is_aligned() {
                return ErrorStatus::InvaildPtr;
            }

            unsafe { core::slice::from_raw_parts_mut($ptr, $len) }
        } else {
            &mut []
        }
    };
}

pub use crate::make_slice;
pub use crate::make_slice_mut;
