# core-os-riscv

An operating system in Rust.

This project is based on "The Adventures of OS: Making a RISC-V Operating System using Rust". In un-modified files, original licenses are preserved.

## Build Instructions

```bash
make qemu
```

## Roadmap

The main goal of this project is to make a xv6-like operating system with the Rust programming language. And now it's in a very early stage. I'm still working on it.

- [x] Adapt code from http://osblog.stephenmarz.com/
- [x] UART drivers
- [x] Virtual Memory
- [x] Load ELF files from memory
- [ ] Process (Currently WIP, I fail to jump into user mode, maybe I'll implement process in supervisor mode)
- [ ] User-mode process
- [ ] Global Allocator
- [ ] Allocator and stdlib
- [ ] Multi-core support
- [ ] System call
- [ ] Persistence
- [ ] Documentation
- [ ] High-level abstractions (driver, vm, etc.)
- [ ] Adapt to aarch64 and deploy on Raspi
- [ ] Rewrite code from other sources
- [ ] Security issues

## Reference

[1] https://github.com/rust-embedded/rust-raspi3-OS-tutorials

[2] https://github.com/bztsrc/raspi3-tutorial/

[3] https://os.phil-opp.com/

[4] http://osblog.stephenmarz.com/

[5] https://github.com/mit-pdos/xv6-riscv/