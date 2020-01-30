TYPE=debug
RELEASE_FLAG=
K=kernel/src
U=user/src
TARGET=riscv64gc-unknown-none-elf
CC=riscv64-unknown-elf-gcc
CFLAGS=-Wall -Wextra -pedantic
CFLAGS+=-static -ffreestanding -nostdlib -fno-rtti -fno-exceptions
CFLAGS+=-march=rv64gc -mabi=lp64
-Wall -Werror -O -fno-omit-frame-pointer -ggdb -MD -mcmodel=medany -ffreestanding -fno-common -nostdlib -mno-relax -I. -fno-stack-protector -fno-pie -no-pie
TARGET_PATH=./target/$(TARGET)/$(TYPE)
KERNEL_LIBS=$(TARGET_PATH)
KERNEL_LIB=-lkernel -lgcc
KERNEL_LINKER_SCRIPT=$K/kernel.ld
KERNEL_LIB_OUT=$(LIBS)/libkernel.a
KERNEL_OUT=kernel.elf
USER_LIB_OUT=$(LIBS)/libuser.rlib
USER_LINKER_SCRIPT=$U/user.ld

QEMU_BINARY=qemu-system-riscv64
MACH=virt
CPU=rv64
CPUS=4
MEM=128M
QEMU_DRIVE=hdd.img

all: $(KERNEL_OUT) $(USER_LIB_OUT)

K_AUTOGEN_FILES = $K/asm/symbols.S $K/symbols_gen.rs $K/syscall_gen.rs
U_AUTOGEN_FILES = $U/usys.S

ASSEMBLY_FILES = $K/asm/boot.S $K/asm/trap.S \
				 $K/asm/trampoline.S $K/asm/symbols.S \
				 $K/asm/swtch.S

$(KERNEL_LIB_OUT): $(K_AUTOGEN_FILES) $(USER_LIB_OUT) $(TARGET_PATH)/init FORCE
	cd kernel && cargo xbuild --target=$(TARGET) $(RELEASE_FLAG)

$(KERNEL_OUT): $(KERNEL_LIB_OUT) $(ASSEMBLY_FILES) $(LINKER_SCRIPT)
	$(CC) $(CFLAGS) -T$(KERNEL_LINKER_SCRIPT) -o $@ $(ASSEMBLY_FILES) -L$(KERNEL_LIBS) $(KERNEL_LIB)

$(USER_LIB_OUT): $(U_AUTOGEN_FILES) FORCE
	cd user && RUSTFLAGS="-C link-arg=-T$(USER_LINKER_SCRIPT)" cargo xbuild --target=$(TARGET) $(RELEASE_FLAG)

$K/asm/symbols.S: utils/symbols.S.py utils/symbols.py
	$< > $@
$K/symbols_gen.rs: utils/symbols_gen.rs.py utils/symbols.py
	$< > $@
$K/syscall_gen.rs: utils/syscall_gen.rs.py utils/syscall.py
	$< > $@
$U/usys.S: utils/usys.S.py utils/syscall.py
	$< > $@

$(QEMU_DRIVE):
	dd if=/dev/zero of=$@ count=32 bs=1048576

qemu: all $(QEMU_DRIVE)
	$(QEMU_BINARY) -machine $(MACH) -cpu $(CPU) -smp $(CPUS) -m $(MEM) \
		-nographic -serial mon:stdio -bios none -kernel $(KERNEL_OUT) \
		-drive if=none,format=raw,file=$(QEMU_DRIVE),id=foo -device virtio-blk-device,drive=foo 

qemunostdio: all $(QEMU_DRIVE)
    $(QEMU_BINARY) -machine $(MACH) -cpu $(CPU) -smp $(CPUS) -m $(MEM) \
            -nographic -serial stdio -bios none -kernel $(KERNEL_OUT) \
            -drive if=none,format=raw,file=$(QEMU_DRIVE),id=foo -device virtio-blk-device,drive=foo

qemudbg: all $(QEMU_DRIVE)
	$(QEMU_BINARY) -machine $(MACH) -cpu $(CPU) -smp $(CPUS) -m $(MEM) \
		-nographic -serial mon:stdio -bios none -kernel $(KERNEL_OUT) \
		-drive if=none,format=raw,file=$(QEMU_DRIVE),id=foo -device virtio-blk-device,drive=foo \
		-d int -d in_asm

qemugdb: all $(QEMU_DRIVE)
	$(QEMU_BINARY) -machine $(MACH) -cpu $(CPU) -smp $(CPUS) -m $(MEM) \
		-nographic -serial mon:stdio -bios none -kernel $(KERNEL_OUT) \
		-drive if=none,format=raw,file=$(QEMU_DRIVE),id=foo -device virtio-blk-device,drive=foo \
		-s

objdump: $(KERNEL_OUT)
	cargo objdump --target $(TARGET) -- -disassemble -no-show-raw-insn -print-imm-hex $(KERNEL_OUT)

readelf: $(KERNEL_OUT)
	readelf -a $<

USERPROG = ./target/$(TARGET)/$(TYPE)/loop

userobjdump: $(USERPROG)
	cargo objdump --target $(TARGET) -- -disassemble -no-show-raw-insn -print-imm-hex $<

userreadelf: $(USERPROG)
	readelf -a $<

.PHONY: clean
clean:
	cargo clean
	rm -f $(KERNEL_OUT) $(OUTPUT)
	rm -f $(K_AUTOGEN_FILES) $(U_AUTOGEN_FILES)
FORCE:
