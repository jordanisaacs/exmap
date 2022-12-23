#![allow(
    non_camel_case_types,
    non_upper_case_globals,
    dead_code,
    non_snake_case,
    unused_qualifications
)]

use rustix::{fd::AsRawFd, io};
use std::ffi::{c_int, c_void};

include!("sys.rs");

fn to_result(ret: c_int) -> io::Result<c_int> {
    if ret >= 0 {
        Ok(ret)
    } else {
        Err(rustix::io::Errno::from_raw_os_error(-ret))
    }
}

#[repr(C)]
#[derive(Debug)]
pub(crate) struct iovec {
    pub(crate) iov_base: *mut c_void,
    pub(crate) iov_len: usize,
}

#[repr(transparent)]
pub(crate) struct exmap_iov_wrapper(ExmapIov);

#[repr(C)]
pub(crate) struct user_interface {}

/// Generates the mmap offset for an interface
///
/// Manually taken from exmap header because C macros
/// are not generated with bindgen
pub const fn EXMAP_OFF_INTERFACE(n: i64) -> i64 {
    EXMAP_OFF_INTERFACE_BASE | (n << 12)
}

pub(crate) unsafe fn exmap_setup<Fd: AsRawFd>(
    fd: &Fd,
    params: &exmap_ioctl_setup,
) -> io::Result<()> {
    to_result(unsafe {
        sc::syscall3(
            sc::nr::IOCTL,
            fd.as_raw_fd() as usize,
            Fix753_EXMAP_IOCTL_SETUP as usize,
            params as *const _ as usize,
        )
    } as _)
    .map(|_| ())
}

// SAFETY:
// The file descriptor is a valid exmap fd. The ioctl request has the correct corresponding
// argument
pub(crate) unsafe fn exmap_ioctl<Fd: AsRawFd>(
    fd: &Fd,
    params: &exmap_action_params,
) -> io::Result<c_int> {
    to_result(unsafe {
        sc::syscall3(
            sc::nr::IOCTL as usize,
            fd.as_raw_fd() as usize,
            Fix753_EXMAP_IOCTL_ACTION as usize,
            params as *const _ as usize,
        )
    } as _)
}
