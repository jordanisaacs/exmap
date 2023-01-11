# Exmap

The exmap kernel module creates a memory area that isn't managed by the Linux kernel.
In that area, memory allocation and freeing, as well as reads and writes can only be done explicitly by the applications using exmap.
One possible use case is the buffer manager of a database.

For further documentation see the `doc/` folder.

## On Nix

Call `nix develop .#` or do `direnv allow`.

To run the vm with the exmap module and bin, call `runvm`. Then inside the qemu vm run the `exmap` command.

To debug the kernel module in a second terminal call `rungdb`. You can type in `lx-symbols-nix` to load in the `exmap.ko` symbols. Then you can use gdb to debug the qemu vm kernel/module.

