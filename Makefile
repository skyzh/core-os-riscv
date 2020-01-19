LINKER_SCRIPT=src/kernel/kernel.ld
RUSTFLAGS=-C link-arg=-T$(LINKER_SCRIPT)
TARGET=riscv64gc-unknown-none-elf
TYPE=release
CARGO_OUTPUT=./target/$(TARGET)/$(TYPE)/kernel
OUTPUT=kernel.img
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

all: $(CARGO_OUTPUT)

AUTOGEN_FILES = src/kernel/asm/symbols.S src/kernel/symbols_gen.rs src/user/usys.S
$(CARGO_OUTPUT): $(AUTOGEN_FILES) $(LINKER_SCRIPT) FORCE
	RUSTFLAGS="$(RUSTFLAGS)" cargo xbuild --target=$(TARGET) --release

# $(OUTPUT): $(CARGO_OUTPUT)
#	$(OBJCOPY_CMD) $< ./$(OUTPUT)

src/kernel/asm/symbols.S: utils/symbols.py utils/symbols.S.py
	./utils/symbols.S.py > $@
src/kernel/symbols_gen.rs: utils/symbols.py utils/symbols_gen.rs.py
	./utils/symbols_gen.rs.py > $@
src/user/usys.S: utils/usys.S.py
	./utils/usys.S.py > $@

$(QEMU_DRIVE):
	dd if=/dev/zero of=$@ count=32 bs=1048576

qemu: $(CARGO_OUTPUT) $(QEMU_DRIVE)
	$(QEMU_BINARY) -machine $(MACH) -cpu $(CPU) -smp $(CPUS) -m $(MEM)  -nographic -serial mon:stdio -bios none -kernel $(CARGO_OUTPUT) -drive if=none,format=raw,file=$(QEMU_DRIVE),id=foo -device virtio-blk-device,drive=foo
	
qemudbg: $(CARGO_OUTPUT) $(QEMU_DRIVE)
	$(QEMU_BINARY) -machine $(MACH) -cpu $(CPU) -smp $(CPUS) -m $(MEM)  -nographic -serial mon:stdio -bios none -kernel $(CARGO_OUTPUT) -drive if=none,format=raw,file=$(QEMU_DRIVE),id=foo -device virtio-blk-device,drive=foo -d int -d in_asm

objdump: $(CARGO_OUTPUT)
	cargo objdump --target $(TARGET) -- -disassemble -no-show-raw-insn -print-imm-hex $(CARGO_OUTPUT)

.PHONY: clean
clean:
	cargo clean
	rm -f $(CARGO_OUTPUT) $(OUTPUT)
FORCE:
