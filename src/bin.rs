use std::{thread::sleep, time::Duration};

use exmap::OwnedExmapFd;
use rustix::{
    fd::{AsFd, AsRawFd, BorrowedFd, FromRawFd, OwnedFd},
    fs::{self, Mode, OFlags},
    io,
};

fn main() {
    let threads = 4;
    let exmap_fd = OwnedExmapFd::<4096>::open().unwrap();
    let exmap = exmap_fd
        .create(
            threads as usize * 4 * 1024 * 1024,
            threads,
            threads as usize * 512,
            None,
        )
        .unwrap();

    let mut interface = unsafe { exmap_fd.mmap_interface(0).unwrap() };
    for i in 0..8 {
        interface.push(i, 1).unwrap();
    }

    interface.push(10, 2).unwrap();
    interface.push(2090, 10).unwrap();

    println!("testing");

    for v in interface.iter() {
        println!("{} {}", v.page(), v.len())
    }

    let (interface, res) = interface.alloc();

    println!("res: {}", res);
    for v in interface.iter() {
        println!("{} {}", v.res, v.pages)
    }

    let mut interface = interface.into_iov();
    for i in 0..5 {
        interface.push(i, 1).unwrap();
    }
    interface.push(1037, 1805).unwrap();
    for v in interface.iter() {
        println!("{} {}", v.page(), v.len())
    }
    let (interface, res) = interface.free();
    println!("res: {}", res);
    for v in interface.iter() {
        println!("{} {}", v.res, v.pages)
    }

    exmap.unmap();
    interface.unmap().unwrap();
    drop(exmap_fd);
}
