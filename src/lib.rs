mod sys;

use std::{
    ffi::c_void,
    marker::PhantomData,
    ops::{Index, IndexMut},
    ptr,
    slice::{Iter, IterMut},
};

use rustix::{
    fd::{AsFd, AsRawFd, BorrowedFd, FromRawFd, OwnedFd},
    fs::{self, Mode, OFlags},
    io,
    mm::{self, MapFlags, ProtFlags},
};
use sys::EXMAP_OFF_INTERFACE;
use thiserror::Error;

pub struct VMMap {
    data: *mut u8,
    len: usize,
}

impl VMMap {
    pub fn unmap(self) {
        println!("unmap vmmap");
        unsafe { mm::munmap(self.data.cast(), self.len) }.unwrap();
    }
}

impl Drop for VMMap {
    fn drop(&mut self) {
    }
}

pub struct InterfaceIov;
pub struct InterfaceResult;

pub struct InterfaceWrapper<'a, T> {
    index: u16,
    data: *mut sys::exmap_user_interface,
    len: u16,
    exmap_fd: BorrowedExmapFd<'a>,
    state: PhantomData<T>,
}

    const MMAP_INTERFACE: usize = std::mem::size_of::<sys::exmap_user_interface>() as usize;

impl<'a, T> InterfaceWrapper<'a, T> {
    pub const MAX_COUNT: usize = sys::EXMAP_USER_INTERFACE_PAGES as usize;

    pub fn unmap(self) -> io::Result<()> {
        println!("drop interface[{}] at {:p}", self.index, self.data);
        unsafe { mm::munmap(self.data as *mut _, MMAP_INTERFACE) }
    }
}

impl<'a> InterfaceWrapper<'a, InterfaceResult> {
    pub fn into_iov(self) -> InterfaceWrapper<'a, InterfaceIov> {
        let InterfaceWrapper {
            index,
            data,
            exmap_fd,
            ..
        } = self;

        InterfaceWrapper {
            index,
            data,
            exmap_fd,
            len: 0,
            state: PhantomData,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &sys::exmap_iov__bindgen_ty_1__bindgen_ty_2> {
        unsafe { &(*self.data).anon1.iov }
            .iter()
            .take(self.len.into())
            .map(|v| unsafe { &v.anon1.anon2 })
    }
}

impl<'a> InterfaceWrapper<'a, InterfaceIov> {

    pub fn alloc(self) -> (InterfaceWrapper<'a, InterfaceResult>, u16) {
        // Result is stored in the memory map
        let res = self.exmap_fd.alloc(self.index, self.len).unwrap();

        (unsafe { self.into_res() }, res)
    }

    pub fn free(self) -> (InterfaceWrapper<'a, InterfaceResult>, u16) {
        // Result is stored in the memory map
        let res = self.exmap_fd.free(self.index, self.len).unwrap();

        (unsafe { self.into_res() }, res)
    }

    unsafe fn into_res(self) -> InterfaceWrapper<'a, InterfaceResult> {
        let InterfaceWrapper {
            index,
            data,
            len,
            exmap_fd,
            ..
        } = self;

        InterfaceWrapper {
            index,
            data,
            exmap_fd,
            len,
            state: PhantomData,
        }
    }

    pub fn push(&mut self, page: u64, len: u64) -> Result<(), ()> {
        if Self::MAX_COUNT == self.len.into() {
            return Err(());
        }

        let l = self.len;
        self.len += 1;
        let x = &mut self[l];
        x.set_len(len);
        x.set_page(page);
        Ok(())
    }

    pub fn iter(&mut self) -> impl Iterator<Item = &sys::ExmapIov> {
        unsafe { &(*self.data).anon1.iov }
            .iter()
            .take(self.len.into())
            .map(|v| unsafe { &v.anon1.anon1 })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut sys::ExmapIov> {
        unsafe { &mut (*self.data).anon1.iov }
            .iter_mut()
            .take(self.len.into())
            .map(|v| unsafe { &mut v.anon1.anon1 })
    }
}

impl<'a, 'b> Index<u16> for InterfaceWrapper<'b, InterfaceIov> {
    type Output = sys::ExmapIov;

    fn index(&self, index: u16) -> &Self::Output {
        if index >= self.len.into() {
            panic!("out of bounds")
        }

        unsafe { &(*self.data).anon1.iov[usize::from(index)].anon1.anon1 }
    }
}

impl<'b> IndexMut<u16> for InterfaceWrapper<'b, InterfaceIov> {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        if index >= self.len.into() {
            panic!("out of bounds")
        }

        unsafe { &mut (*self.data).anon1.iov[usize::from(index)].anon1.anon1 }
    }
}

#[derive(Error, Debug)]
pub enum InterfaceIndexError {
    #[error("index is out of bounds")]
    OutOfBounds,
}

#[derive(Debug)]
pub struct OwnedExmapFd<const PAGE_SIZE: usize>(OwnedFd);

impl<const PAGE_SIZE: usize> OwnedExmapFd<PAGE_SIZE> {
    pub fn open() -> io::Result<OwnedExmapFd<PAGE_SIZE>> {
        let fd = fs::openat(fs::cwd(), "/dev/exmap", OFlags::RDWR, Mode::empty())?;
        Ok(OwnedExmapFd(fd))
    }

    #[inline]
    fn _mmap(&self, length: usize, offset: u64) -> io::Result<*mut c_void> {
        let prot: ProtFlags = ProtFlags::READ | ProtFlags::WRITE;
        let flags: MapFlags = MapFlags::SHARED;

        // SAFETY:
        // Passing null pointer so do not need to deal with alignment
        unsafe { mm::mmap(ptr::null_mut(), length, prot, flags, &self.0, offset) }
    }

    fn mmap_vm(&self, size: usize) -> io::Result<VMMap> {
        let data = self._mmap(size, sys::EXMAP_OFF_EXMAP.into())? as *mut u8;
        Ok(VMMap { data, len: size })
    }

    /// Safety: Can only map an interface value once.
    pub unsafe fn mmap_interface(
        &self,
        index: u16,
    ) -> io::Result<InterfaceWrapper<'_, InterfaceIov>> {
        let interface_num = EXMAP_OFF_INTERFACE(index.into()) as u64;
        let data = self._mmap(MMAP_INTERFACE, interface_num)?
            as *mut sys::exmap_user_interface;

        println!(
            "mmap interface[{}] at address {:#X}",
            interface_num, data as usize
        );

        Ok(InterfaceWrapper {
            data,
            len: 0,
            exmap_fd: self.as_fd(),
            index,
            state: PhantomData,
        })
    }

    fn setup<'b>(
        &self,
        backing_fd: Option<BorrowedFd<'b>>,
        max_interfaces: u16,
        buffer_size: usize,
    ) -> io::Result<()> {
        // If there is no backing fd, then exmap expects -1
        let backing_fd_raw = if let Some(fd) = backing_fd {
            fd.as_raw_fd()
        } else {
            -1
        };

        let params = sys::exmap_ioctl_setup {
            fd: backing_fd_raw,
            max_interfaces: max_interfaces.into(),
            buffer_size,
            flags: 0, // Not currently used by exmap
        };

        unsafe { sys::exmap_setup(&self.0, &params) }
    }

    /// Size of the virtual mmeory ofr exmap
    /// Max number of interfaces
    /// Amount of memory reserved for the exmap
    /// Optional backing file descriptor
    pub fn create<'a, 'b: 'a>(
        &'a self,
        exmap_size: usize,
        max_interfaces: u16,
        buffer_size: usize,
        backing_fd: Option<BorrowedFd<'b>>,
    ) -> io::Result<VirtMem<'a, 'b, PAGE_SIZE>> {
        assert!(MMAP_INTERFACE <= PAGE_SIZE);

        // Initialize the exmap vma with its size
        let vmmap = self.mmap_vm(exmap_size)?;

        // Configure exmap
        let _ = self.setup(backing_fd, max_interfaces, buffer_size)?;

        Ok(VirtMem {
            exmap_fd: self.as_fd(),
            vmmap,
            backing_fd,
        })
    }

    fn as_fd(&self) -> BorrowedExmapFd<'_> {
        BorrowedExmapFd(self.0.as_fd())
    }
}

impl<const PAGE_SIZE: usize> Drop for OwnedExmapFd<PAGE_SIZE> {
    fn drop(&mut self) {
        println!("Dropping file descriptor");
    }
}

impl<const PAGE_SIZE: usize> FromRawFd for OwnedExmapFd<PAGE_SIZE> {
    unsafe fn from_raw_fd(fd: rustix::fd::RawFd) -> OwnedExmapFd<PAGE_SIZE> {
        unsafe { OwnedExmapFd(OwnedFd::from_raw_fd(fd)) }
    }
}

#[derive(Debug)]
pub struct BorrowedExmapFd<'a>(BorrowedFd<'a>);

impl<'a> BorrowedExmapFd<'a> {
    fn alloc(&self, interface: u16, iov_len: u16) -> io::Result<u16> {
        let params = sys::exmap_action_params {
            interface,
            iov_len,
            opcode: sys::EXMAP_OP_ALLOC as u16,
            flags: 0, // TODO: Figure out flag situation
        };

        unsafe { sys::exmap_ioctl(&self.0, &params).map(|c| c as u16) }
    }

    fn free(&self, interface: u16, iov_len: u16) -> io::Result<u16> {
        let params = sys::exmap_action_params {
            interface,
            iov_len,
            opcode: sys::EXMAP_OP_FREE as u16,
            flags: 0, // TODO: Figure out flag situation
        };

        unsafe { sys::exmap_ioctl(&self.0, &params).map(|c| c as u16) }
    }
}

pub struct VirtMem<'a, 'b, const PAGE_SIZE: usize> {
    exmap_fd: BorrowedExmapFd<'a>,
    backing_fd: Option<BorrowedFd<'b>>,
    vmmap: VMMap,
}

impl<'a, 'b, const P: usize> VirtMem<'a, 'b, P> {
    pub fn read() {
        todo!()
    }

    pub fn readv() {
        todo!()
    }

    pub fn unmap(self) {
        self.vmmap.unmap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mmap_vm_fail() {
        let exmap_fd = OwnedExmapFd::<4096>::open().unwrap();

        let _ = exmap_fd.mmap_vm(2048).unwrap();

        assert!(exmap_fd.mmap_vm(2048).is_err());
    }

    #[test]
    fn it_works() {
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
    }
}
