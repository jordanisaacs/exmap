{
  linuxKernel,
  lib,
}: let
  version = "6.1";
  localVersion = "-development";
  src = linuxKernel.packages.linux_6_1.kernel.src;
in {
  kernelArgs = {
    inherit version src;

    inherit localVersion;
    modDirVersion = let
      appendV = ".0";
    in
      version + appendV + localVersion;
  };

  kernelConfig = {
    # See https://github.com/NixOS/nixpkgs/blob/master/nixos/modules/system/boot/kernel_config.nix
    structuredExtraConfig = with lib.kernel; {
      # REQUIRED: Need MMU_NOTIFIER for exmap module
      VIRTUALIZATION = yes;
      HIGH_RES_TIMERS = yes;
      SMP = yes;
      KVM = yes; # Enabled by above

      DEBUG_FS = yes;
      DEBUG_KERNEL = yes;
      DEBUG_MISC = yes;
      DEBUG_BUGVERBOSE = yes;
      DEBUG_BOOT_PARAMS = yes;
      DEBUG_STACK_USAGE = yes;
      DEBUG_SHIRQ = yes;
      DEBUG_ATOMIC_SLEEP = yes;

      DEBUG_INFO_DWARF_TOOLCHAIN_DEFAULT = yes;
      GDB_SCRIPTS = yes;

      IKCONFIG = yes;
      IKCONFIG_PROC = yes;
      # Compile with headers
      IKHEADERS = yes;

      SLUB_DEBUG = yes;
      DEBUG_MEMORY_INIT = yes;
      KASAN = yes;

      # FRAME_WARN - warn at build time for stack frames larger tahn this.

      MAGIC_SYSRQ = yes;

      LOCALVERSION = freeform localVersion;

      LOCK_STAT = yes;
      PROVE_LOCKING = yes;

      FTRACE = yes;
      STACKTRACE = yes;
      IRQSOFF_TRACER = yes;

      KGDB = yes;
      UBSAN = yes;
      BUG_ON_DATA_CORRUPTION = yes;
      SCHED_STACK_END_CHECK = yes;
      FRAME_POINTER = yes;
      UNWINDER_FRAME_POINTER = yes;
      "64BIT" = yes;

      # initramfs/initrd ssupport
      BLK_DEV_INITRD = yes;

      # https://docs.kernel.org/admin-guide/dynamic-debug-howto.html
      DYNAMIC_DEBUG = yes;
      PRINTK = yes;
      PRINTK_TIME = yes;
      PRINTK_CALLER = yes;
      EARLY_PRINTK = yes;

      # Support elf and #! scripts
      BINFMT_ELF = yes;
      BINFMT_SCRIPT = yes;

      # Create a tmpfs/ramfs early at bootup.
      DEVTMPFS = yes;
      DEVTMPFS_MOUNT = yes;

      TTY = yes;
      SERIAL_8250 = yes;
      SERIAL_8250_CONSOLE = yes;

      PROC_FS = yes;
      SYSFS = yes;

      # insmod/rmmod modules
      MODULES = yes;
      MODULE_UNLOAD = yes;

      KPROBES = yes;
      KALLSYMS_ALL = yes;
    };

    # Flags that get passed to generate-config.pl
    generateConfigFlags = {
      # Ignores any config errors (eg unused config options)
      ignoreConfigErrors = false;
      # Build every available module
      autoModules = false;
      preferBuiltin = false;
    };
  };
}
