mod sys;

use std::{
    fs::OpenOptions,
    marker::PhantomData,
    os::unix::prelude::{AsRawFd, BorrowedFd, OwnedFd},
};

use sys::exmap_setup;

pub struct Exmap<'a> {
    _exmap_fd: OwnedFd,
    backing_fd: PhantomData<&'a ()>,
}

impl<'a> Exmap<'a> {
    pub fn setup<'b: 'a>(
        max_interfaces: u32,
        buffer_size: u32,
        backing_fd: Option<BorrowedFd<'b>>,
    ) -> Exmap<'a> {
        // If there is no backing fd, then exmap expects -1
        let backing_fd_raw = if let Some(fd) = backing_fd {
            fd.as_raw_fd()
        } else {
            -1
        };

        let exmap_fd: OwnedFd = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/exmap")
            .unwrap()
            .into();

        let setup_params = sys::exmap_ioctl_setup {
            fd: backing_fd_raw,
            max_interfaces: max_interfaces as _,
            buffer_size: buffer_size as _,
            flags: 0,
        };

        let _ = unsafe { exmap_setup(exmap_fd.as_raw_fd(), &setup_params as *const _ as *const _) }
            .unwrap();

        Exmap {
            _exmap_fd: exmap_fd,
            backing_fd: PhantomData,
        }
    }
}

// use std::sync::atomic::{AtomicU64, Ordering};
//
// pub fn add(left: usize, right: usize) -> usize {
//     left + right
// }
//
// const DIRTY_BITS: u64 = 0b1;
// const DIRTY_SHIFT: u64 = VERSION_BITS.count_ones() as u64;
// const DIRTY_MASK: u64 = DIRTY_BITS << DIRTY_SHIFT;
// const CLEAN: u64 = 0;
// const DIRTY: u64 = 1;
//
// const STATE_BITS: u64 = PageStateType::Evicted as u64;
// const STATE_SHIFT: u64 = (VERSION_BITS.count_ones() + DIRTY_BITS.count_ones()) as u64;
// const STATE_MASK: u64 = STATE_BITS << STATE_SHIFT;
//
// const VERSION_BITS: u64 = u64::MAX >> (STATE_BITS.count_ones() + DIRTY_BITS.count_ones());
// const VERSION_MASK: u64 = VERSION_BITS;
//
// enum PageStateType {
//     Unlocked = 0,
//     LockedShared = 252,
//     UnlockedShared = 253,
//     Marked = 254,
//     Evicted = 255,
// }
//
// struct StateEntry(AtomicU64);
//
// impl StateEntry {
//     const fn init() -> Self {
//         let v = (CLEAN << DIRTY_SHIFT) | ((PageStateType::Evicted as u64) << STATE_SHIFT);
//         StateEntry(AtomicU64::new(v))
//     }
//
//     fn mark_dirty(&self) -> Result<u64, u64> {
//         let old_state = self.0.load(Ordering::Acquire);
//         let new_state = old_state & (DIRTY << DIRTY_SHIFT);
//         return self
//             .0
//             .compare_exchange(old_state, new_state, Ordering::AcqRel, Ordering::Acquire);
//     }
//
//     fn mark_clean(&self) -> Result<u64, u64> {
//         let old_state = self.0.load(Ordering::Acquire);
//         let new_state = old_state & (CLEAN << DIRTY_SHIFT);
//         return self
//             .0
//             .compare_exchange(old_state, new_state, Ordering::AcqRel, Ordering::Acquire);
//     }
// }
//

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let _ = Exmap::setup(258, 1024, None);
    }
}
