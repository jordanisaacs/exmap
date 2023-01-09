mod sys;

use std::{ffi::c_void, ops::Index, ptr, slice::Iter};

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

impl VMMap {}

impl Drop for VMMap {
    fn drop(&mut self) {
        println!("unmap vmmap");
        unsafe { mm::munmap(self.data.cast(), self.len) }.unwrap();
    }
}

#[derive(Error, Debug)]
pub enum InterfaceIndexError {
    #[error("index is out of bounds")]
    OutOfBounds,
}

pub struct Interface<'b> {
    data: *mut sys::exmap_user_interface,
    index: u16,
    exmap_fd: BorrowedExmapFd<'b>,
}

impl<'b> Interface<'b> {
    pub const COUNT: usize = sys::EXMAP_USER_INTERFACE_PAGES as usize;
    const SIZE: usize = std::mem::size_of::<sys::exmap_user_interface>() as usize;

    pub fn get_mut(&mut self, index: usize) -> Result<InterfaceIovMut<'_>, InterfaceIndexError> {
        let Some(iov_union) = (unsafe { (*self.data).anon1.iov.get_mut(index) })
         else {
             return Err(InterfaceIndexError::OutOfBounds);
         };

        let iov = unsafe { &mut iov_union.anon1.anon1 };
        Ok(InterfaceIovMut(iov))
    }

    pub fn get(&self, index: usize) -> Result<InterfaceIov<'_>, InterfaceIndexError> {
        let Some(iov_union) = (unsafe { (*self.data).anon1.iov.get(index) })
         else {
             return Err(InterfaceIndexError::OutOfBounds);
         };

        let iov = unsafe { &iov_union.anon1.anon1 };
        Ok(InterfaceIov(iov))
    }

    pub fn alloc(&self, iov_len: u16) -> io::Result<u16> {
        self.exmap_fd.alloc(self.index, iov_len)
    }

    pub fn iter(&self) -> impl Iterator<Item = InterfaceIov<'_>> {
        unsafe { &(*self.data).anon1.iov }
            .iter()
            .map(|iov| InterfaceIov(unsafe { &iov.anon1.anon1 }))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = InterfaceIovMut<'_>> {
        unsafe { &mut (*self.data).anon1.iov }
            .iter_mut()
            .map(|iov| InterfaceIovMut(unsafe { &mut iov.anon1.anon1 }))
    }

    pub fn print(&self, index: usize) {
        println!("start address: {:p}", self.data);
        for i in 0..index {
            let v = unsafe { (*self.data).anon1.iov.as_ref().index(i) };
            let o = unsafe { &v.anon1.anon1 };
            println!("address {} {:p}: {} {}", i, v, o.page(), o.len());
        }
    }
}

impl<'b> Drop for Interface<'b> {
    fn drop(&mut self) {
        println!("drop interface[{}] at {:p}", self.index, self.data);
        unsafe { mm::munmap(self.data as *mut _, Self::SIZE) }.unwrap();
    }
}

pub struct Iovec {}

pub struct InterfaceIovMut<'a>(&'a mut sys::ExmapIov);
impl<'a> InterfaceIovMut<'a> {
    /// Set the starting page of the iov.
    ///
    /// The address is within the exmap's address space and should be aligned to page size
    pub fn set_page(&mut self, val: u64) {
        self.0.set_page(val);
    }

    /// Set the length in page of the iov
    pub fn set_len(&mut self, val: u64) {
        self.0.set_len(val);
    }

    /// Get the starting address of the iov
    pub fn page(&self) -> u64 {
        self.0.page()
    }

    /// Set the lenght of the iov in terms of pages
    pub fn len(&self) -> u64 {
        self.0.len()
    }
}

pub struct InterfaceIov<'a>(&'a sys::ExmapIov);
impl<'a> InterfaceIov<'a> {
    pub fn page(&self) -> u64 {
        self.0.page()
    }

    pub fn len(&self) -> u64 {
        self.0.len()
    }
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

    pub fn mmap_interface(&self, index: u16) -> io::Result<Interface<'_>> {
        let interface_num = EXMAP_OFF_INTERFACE(index.into()) as u64;
        let data = self._mmap(Interface::SIZE, interface_num)? as *mut sys::exmap_user_interface;

        println!(
            "mmap interface[{}] at address {:#X}",
            interface_num, data as usize
        );

        Ok(Interface {
            data,
            exmap_fd: self.as_fd(),
            index,
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
        assert!(Interface::SIZE <= PAGE_SIZE);

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

        let mut interface = exmap_fd.mmap_interface(0).unwrap();
        for (i, mut v) in interface.iter_mut().enumerate() {
            v.set_page(i as u64);
            v.set_len(1);
        }

        println!("{}", interface.alloc(0).unwrap());
        println!("{}", interface.alloc(10).unwrap());
        println!("{}", interface.alloc(10).unwrap());

        // Allocate interfaces
        // let mut interfaces = Vec::with_capacity(threads as usize);
        // for i in 0..Interface::SIZE {
        //     let mut iov = interfaces[0].index_mut(i);
        //     iov.set_page(i as u64);
        //     iov.set_len(1);
        // }
    }
}
