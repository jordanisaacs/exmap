all: modules

# out-of-tree build for our kernel-module, firmware and inmates
KDIR = $(KERNEL)/lib/modules/$(KERNEL_VERSION)/build

obj-m = exmap.o
exmap-m = driver.o memory.o ksyms.o
ccflags-y += -g 

kbuild = -C $(KDIR) M=$$PWD $@

modules:
	$(Q)$(MAKE) $(kbuild)

clean:
	$(Q)$(MAKE) $(kbuild)
