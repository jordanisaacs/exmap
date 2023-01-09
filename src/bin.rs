use exmap::OwnedExmapFd;
use rustix::{
    fd::{AsFd, AsRawFd, BorrowedFd, FromRawFd, OwnedFd},
    fs::{self, Mode, OFlags},
    io,
};

fn main() {
    let interface_vec = 2;
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

    let mut interface = exmap_fd.mmap_interface(0).unwrap();
    for (i, mut v) in interface.iter_mut().enumerate() {
        v.set_page(i as u64);
        v.set_len(1);
    }
    interface.print(5);

    interface.alloc(5);


    // println!("{}", interface.alloc(0).unwrap());
    // println!("{}", interface.alloc(10).unwrap());
    // println!("{}", interface.alloc(10).unwrap());
}
