LINKER_SCRIPT=-Tsrc/kernel.ld
RUSTFLAGS=-C link-arg=$(LINKER_SCRIPT)
TARGET=riscv64gc-unknown-none-elf
TYPE=release
CARGO_OUTPUT=./target/$(TARGET)/$(TYPE)/core-os-riscv
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

$(CARGO_OUTPUT): FORCE
	RUSTFLAGS="$(RUSTFLAGS)" cargo xbuild --target=$(TARGET) --release

# $(OUTPUT): $(CARGO_OUTPUT)
#	$(OBJCOPY_CMD) $< ./$(OUTPUT)

$(QEMU_DRIVE):
	dd if=/dev/zero of=$@ count=32 bs=1048576

qemu: $(CARGO_OUTPUT) $(QEMU_DRIVE)
	$(QEMU_BINARY) -machine $(MACH) -cpu $(CPU) -smp $(CPUS) -m $(MEM)  -nographic -serial mon:stdio -bios none -kernel $(CARGO_OUTPUT) -drive if=none,format=raw,file=$(QEMU_DRIVE),id=foo -device virtio-blk-device,drive=foo
	
qemudbg: $(CARGO_OUTPUT) $(QEMU_DRIVE)
	$(QEMU_BINARY) -machine $(MACH) -cpu $(CPU) -smp $(CPUS) -m $(MEM)  -nographic -serial mon:stdio -bios none -kernel $(CARGO_OUTPUT) -drive if=none,format=raw,file=$(QEMU_DRIVE),id=foo -device virtio-blk-device,drive=foo -d int

objdump: $(CARGO_OUTPUT)
	cargo objdump --target $(TARGET) -- -disassemble -no-show-raw-insn -print-imm-hex $(CARGO_OUTPUT)

objdumpuser: $(CARGO_OUTPUT)
	cargo objdump --target $(TARGET) -- -disassemble -no-show-raw-insn -print-imm-hex ./target/$(TARGET)/$(TYPE)/loop

.PHONY: clean
clean:
	cargo clean
	rm -f $(CARGO_OUTPUT) $(OUTPUT)
FORCE:
