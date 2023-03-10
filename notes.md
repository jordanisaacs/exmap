# Setup


## Open

First open "/dev/exmap" with O_RDWR.

## Create virtual memory mapping

Next need to `mmap` the exmap fd with `EXMAP_OFF_EXMAP` as the offset.
It opens the `vma` and returns a pointer to the virtual memory area.
Attempting to `mmap` the exmap fd a second time will fail.

The `vma` is of type [VM_MIXEDMAP](https://sourcegraph.com/github.com/torvalds/linux/-/commit/b379d790197cdf8a95fb67507d75a24ac0a1678d?visible=2).

> VM_MIXEDMAP is a mapping which supports COW mappings on arbitrary ranges including ranges without struct page *and* ranges with a struct page that we actually want to refcount.
> VM_MIXEDMAP achieves this by refcounting all pfn_valid pages, and not refcounting !pfn_valid pages

## Setup parameters

While there is now have a virtual memory area, nothing can be done with it. It needs to be configured. This is done using `EXMAP_IOCTL_SETUP` ioctl.

The configuration is a struct:

```c
struct exmap_ioctl_setup {
  int    fd;
  int    max_interfaces;
  size_t buffer_size;
  uint64_t flags;
};
```

*fd*: The fd is an optional backing fd. The exmap fd will act as a proxy to the backing fd for reads. Writes however should still use the backing fd.
*max_interfaces*: The max number of interfaces to use. Each interface has its own local free list to get pages from. Thus, it is best to set the number of interfaces to your number of threads (and keep them thread local).
*buffer_size*: This is the max amount of memory that should be used by exmap. It is specified in terms of page size. So a buffer_size of 1000 would be 4MB (assuming a 4KB page size).
*flags*: Not currently used by the setup ioctl

## Mapping Interfaces

With the exmap set up, now we need to setup the interfaces. This is done by mapping `exmap_user_interface` structs. To specify what interface number you are mapping, pass the index into the mmap `offset` parameter. Make sure to wrap the index in the `EXMAP_OFF_INTERFACE` const fn/macro. The mmap call will return the `struct exmap_user_interface`. The struct is a page size (so 4KB). It is an array of 512 (defined by `EXMAP_USER_INTERFACE_PAGES`) `exmap_iov. The struct is used to determine what pages are going to be allocated/free'd. After an ioctl call, detailed results are stored in the mmap'd struct.

```c
struct exmap_user_interface {
  union {
    struct exmap_iov iov[EXMAP_USER_INTERFACE_PAGES];
  };
};
```

An exmap_iov is a union. Arguments for ioctls are:

```c
{
  uint64_t page   : 64 - EXMAP_PAGE_LEN_BITS;
  uint64_t len    : EXMAP_PAGE_LEN_BITS;
};
```

The `page` field is the starting page of entry.
The `len` field is the number of pages to alloc/free from the start page.

The results are in:

```c
{
    int32_t   res;
    int16_t   pages;
};
```

## Exmap ioctl Actions

Acting on an exmap works through through the ioctl or read syscalls. Starting with the ioctl, they take the follow struct as their parameter:

```c
struct exmap_action_params {
	uint16_t interface;
	uint16_t iov_len;
	uint16_t opcode; // exmap_opcode
	uint64_t flags;  // exmap_flags
};
```

## EXMAP_OP_ALLOC

The first operation is the `EXMAP_OP_ALLOC`. `iov_len` determines how many pages we want to allocate. It will first attempt to allocate as many pages as possible *from the system*. As pages are allocated when needed, we may still have space to allocate up to `buffer_size` pages. These pages are pushed into the free list.

After the page memory allocations, the actual pages are made available to user space. This works by reading `exmap_user_interface` associated with the interface passed through the `exmap_action_params`. It allocates pages at address `page * PAGE_SIZE` of length `len * PAGE_SIZE`. Only first `iov_len` entries of the `exmap_user_interface` array will be allocated.

It uses the modified page table structure (atomic exchange for PTE) specified in the paper.

## EXMAP_OP_FREE

Freeing does not do any page allocations (in my branch of the module). It walks down the page table skipping any sections that do not already exist. If it finds an allocated PTE in the memory address range to be freed, it unmaps it and stores it in the interface free list.
