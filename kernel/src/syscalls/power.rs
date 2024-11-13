use crate::{
    arch::power::{reboot, shutdown},
    utils::errors::ErrorStatus,
};

#[no_mangle]
extern "C" fn sysshutdown() -> ErrorStatus {
    shutdown();
    ErrorStatus::None
}

#[no_mangle]
extern "C" fn sysreboot() -> ErrorStatus {
    reboot();
    ErrorStatus::None
}
