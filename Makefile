K=src/kernel
U=src/user
TARGET=riscv64gc-unknown-none-elf
CC=riscv64-unknown-elf-gcc
CFLAGS=-Wall -Wextra -pedantic
CFLAGS+=-static -ffreestanding -nostdlib -fno-rtti -fno-exceptions
CFLAGS+=-march=rv64gc -mabi=lp64
-Wall -Werror -O -fno-omit-frame-pointer -ggdb -MD -mcmodel=medany -ffreestanding -fno-common -nostdlib -mno-relax -I. -fno-stack-protector -fno-pie -no-pie
LIBS=./target/$(TARGET)/$(TYPE)
LIB=-lkernel -lgcc
LINKER_SCRIPT=$K/kernel.ld
KERNEL_LIB=$(LIBS)/libkernel.rlib
KERNEL_OUT=kernel.elf

TYPE=debug
RELEASE_FLAG=
CFLAGS+=-O0 -g

OBJCOPY_CMD = cargo objcopy \
		-- \
		--strip-all \
		-O binary

QEMU_BINARY=qemu-system-riscv64
MACH=virt
CPU=rv64
CPUS=4
MEM=128M
QEMU_DRIVE=hdd.img

all: $(KERNEL_OUT)

AUTOGEN_FILES = $K/asm/symbols.S $K/symbols_gen.rs \
				$U/usys.S

ASSEMBLY_FILES = $K/asm/boot.S $K/asm/trap.S \
				 $K/asm/trampoline.S $K/asm/symbols.S

$(KERNEL_LIB): $(AUTOGEN_FILES) FORCE
	cargo xbuild --target=$(TARGET) $(RELEASE_FLAG)

$(KERNEL_OUT): $(KERNEL_LIB) $(ASSEMBLY_FILES) $(LINKER_SCRIPT)
	$(CC) $(CFLAGS) -T$(LINKER_SCRIPT) -o $@ $(ASSEMBLY_FILES) -L$(LIBS) $(LIB)

# $(OUTPUT): $(KERNEL_OUT)
#	$(OBJCOPY_CMD) $< ./$(OUTPUT)

$K/asm/symbols.S: utils/symbols.py utils/symbols.S.py
	./utils/symbols.S.py > $@
$K/symbols_gen.rs: utils/symbols.py utils/symbols_gen.rs.py
	./utils/symbols_gen.rs.py > $@
$U/usys.S: utils/usys.S.py
	./utils/usys.S.py > $@

user: $U/loop.rs
	cargo rustc $< --target=$(TARGET) $(RELEASE_FLAG)

$(QEMU_DRIVE):
	dd if=/dev/zero of=$@ count=32 bs=1048576

qemu: $(KERNEL_OUT) $(QEMU_DRIVE)
	$(QEMU_BINARY) -machine $(MACH) -cpu $(CPU) -smp $(CPUS) -m $(MEM)  -nographic -serial mon:stdio -bios none -kernel $(KERNEL_OUT) -drive if=none,format=raw,file=$(QEMU_DRIVE),id=foo -device virtio-blk-device,drive=foo -d int
	
qemudbg: $(KERNEL_OUT) $(QEMU_DRIVE)
	$(QEMU_BINARY) -machine $(MACH) -cpu $(CPU) -smp $(CPUS) -m $(MEM)  -nographic -serial mon:stdio -bios none -kernel $(KERNEL_OUT) -drive if=none,format=raw,file=$(QEMU_DRIVE),id=foo -device virtio-blk-device,drive=foo -d int -d in_asm

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
	rm -f $(AUTOGEN_FILES)
FORCE:
