#![allow(
    non_camel_case_types,
    non_upper_case_globals,
    dead_code,
    non_snake_case,
    unused_qualifications
)]

use std::{
    ffi::{c_int, c_long, c_void},
    io,
};

include!("sys.rs");

fn to_result(ret: c_int) -> io::Result<c_int> {
    if ret >= 0 {
        Ok(ret)
    } else {
        Err(io::Error::from_raw_os_error(-ret))
    }
}

const SYSCALL_IOCTL: c_long = __NR_ioctl as _;

pub unsafe fn exmap_setup(fd: c_int, arg: *const c_void) -> io::Result<c_int> {
    to_result(sc::syscall3(
        SYSCALL_IOCTL as usize,
        fd as usize,
        EXMAP_IOCTL_SETUP as usize,
        arg as usize,
    ) as _)
}

pub unsafe fn exmap_action(fd: c_int, arg: *const c_void) -> io::Result<c_int> {
    to_result(sc::syscall3(
        SYSCALL_IOCTL as usize,
        fd as usize,
        EXMAP_IOCTL_ACTION as usize,
        arg as usize,
    ) as _)
}
