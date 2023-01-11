use std::{mem::MaybeUninit, sync::atomic::AtomicU64};

use exmap::OwnedExmapFd;

#[derive(Debug)]
pub enum PageStatus {
    Unlocked,
    LockedShared(u8),
    Locked,
    Marked,
    Evicted,
}

impl Into<u8> for PageStatus {
    fn into(self) -> u8 {
        match self {
            Self::Unlocked => 0,
            Self::Locked => 253,
            Self::Marked => 254,
            Self::Evicted => 255,
            Self::LockedShared(v) => v,
        }
    }
}

impl From<u8> for PageStatus {
    fn from(v: u8) -> Self {
        match v {
            0 => Self::Unlocked,
            253 => Self::Locked,
            254 => Self::Marked,
            255 => Self::Evicted,
            v => Self::LockedShared(v),
        }
    }
}

#[derive(Clone, Copy)]
pub struct PageState {
    data: [u8; 8],
}

impl PageState {
    pub fn new() -> Self {
        Self { data: [0; 8] }
    }
    pub fn version(&self) -> u64 {
        u64::from_le_bytes([
            self.data[0],
            self.data[1],
            self.data[2],
            self.data[3],
            self.data[4],
            self.data[5],
            self.data[6],
            0,
        ])
    }

    pub fn set_version(&mut self, new_val: u64) {
        assert!(new_val < (0x01_u64 << 56));
        let le_bytes = new_val.to_le_bytes();
        self.data[..7].copy_from_slice(&le_bytes[..7])
    }

    pub fn status(&self) -> PageStatus {
        u8::from_le_bytes([self.data[8]]).into()
    }

    pub fn set_status(&mut self, new_val: PageStatus) {
        let v: u8 = new_val.into();
        self.data[7] = v.to_le_bytes()[0]
    }
}

impl From<u64> for PageState {
    fn from(v: u64) -> Self {
        Self {
            data: v.to_le_bytes(),
        }
    }
}

impl Into<u64> for PageState {
    fn into(self) -> u64 {
        u64::from_le_bytes(self.data)
    }
}

struct VMCache<const S: usize> {
    entries: Box<[AtomicU64; S]>,
}

impl<const S: usize> VMCache<S> {
    pub fn new() -> VMCache<S> {
        let init = 0;

        let entries = {
            let mut entries: Box<[MaybeUninit<AtomicU64>; S]> =
                Box::new(unsafe { MaybeUninit::uninit().assume_init() });

            for entry in entries.iter_mut() {
                entry.write(AtomicU64::new(init));
            }

            unsafe { std::mem::transmute::<_, Box<[AtomicU64; S]>>(entries) }
        };

        VMCache { entries }
    }

    // fn fix_multiple<const P: usize>(&self, mut interface: InterfaceWrapper<InterfaceIov>) {
    //     // Deadlock prone
    //     let mut miss_length = 0;
    //     let mut overwrite_prev_iov = false;

    //     for i in 0..interface.len() {
    //         let pid_start = interface[i].page();
    //         let len = interface[i].len();

    //         if overwrite_prev_iov {
    //             interface[miss_length].set_page(pid_start);
    //             interface[miss_length].set_len(len)
    //         }

    //         for pid in pid_start..pid_start + len {
    //             loop {
    //                 let state = self.entries.get(pid as usize).unwrap();

    //                 let curr_state = state.load(Ordering::Relaxed);
    //                 let mut legible_state = PageState::from(curr_state);

    //                 match legible_state.status() {
    //                     PageStatus::Evicted => {
    //                         legible_state.set_status(PageStatus::Locked);
    //                         if state
    //                             .compare_exchange(
    //                                 curr_state,
    //                                 legible_state.into(),
    //                                 Ordering::Relaxed,
    //                                 Ordering::Relaxed,
    //                             )
    //                             .is_ok()
    //                         {
    //                             miss_length += 1;
    //                             break;
    //                         };
    //                     }
    //                     PageStatus::Marked | PageStatus::Unlocked => {
    //                         legible_state.set_status(PageStatus::Locked);
    //                         if state
    //                             .compare_exchange(
    //                                 curr_state,
    //                                 legible_state.into(),
    //                                 Ordering::Relaxed,
    //                                 Ordering::Relaxed,
    //                             )
    //                             .is_ok()
    //                         {
    //                             overwrite_prev_iov = true;
    //                             break;
    //                         };
    //                     }
    //                     _ => continue,
    //                 }
    //             }
    //         }
    //     }
    // }
}

fn main() {
    let threads = 4;
    let exmap_fd = OwnedExmapFd::<4096>::open().unwrap();
    let mut exmap = exmap_fd
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
    interface.push(4095, 1).unwrap();

    println!("testing");

    for v in interface.iter() {
        println!("{} {}", v.page(), v.len())
    }

    let (interface, res) = interface.alloc();

    println!("res: {}", res);
    for v in interface.iter() {
        println!("{} {}", v.res, v.pages)
    }

    let size = exmap.size();
    let x = exmap.as_mut();
    x[0] = 3;
    x[size - 1] = 10;

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
