# core-os-riscv

[![Build Status](https://travis-ci.com/skyzh/core-os-riscv.svg?branch=master)](https://travis-ci.com/skyzh/core-os-riscv) (docs)

An xv6-like operating system in Rust.

This project is originally based on "The Adventures of OS: Making a RISC-V Operating System using Rust",
and is now being made to have an xv6-like structure.

You may browse this repo with [Sourcegraph](https://sourcegraph.com/github.com/skyzh/core-os-riscv).

## Build Instructions

### macOS

First of all, install GNU RISC-V tools and QEMU. Python3 is also required to generate some files automatically.

```bash
brew tap riscv/riscv
brew install riscv-tools
brew test riscv-tools
brew install qemu
```

Don't forget to add riscv-tools to PATH.

Then, install Rust and related components.

```bash
cargo default nightly
cargo install cargo-xbuild cargo-binutils
rustup component add rust-src llvm-tools-preview rustfmt rls rust-analysis
rustup target add riscv64gc-unknown-none-elf
```

Finally you may build and run this project.

```bash
make qemu
```

If you want to use readelf tools, etc., you may install pwntools on macOS.

### Ubuntu

Use Linuxbrew.

## Documentation

Documentation of this repo is automatically built and deployed with Travis. You may view online version [here](https://skyzh.github.io/core-os-riscv/kernel/).

```bash
make docs
```

I'll continuously add Rust-specific implementations and how I made this project into documentation.

## Roadmap

The main goal of this project is to make an xv6-like operating system with the Rust programming language. And now it's in a very early stage. I'm still working on it.

- [x] Adapt code from http://osblog.stephenmarz.com/

* Virtual Memory and Management
    - [x] Virtual Memory
    - [x] Load ELF files from memory
    - [x] Kernel Allocator
    - [x] Remove direct call to allocator
    - [ ] (WIP, Bug) Kernel stack management and mapping
* Traps and Interrupt, Drivers
    - [x] UART drivers
    - [x] Machine-mode Timer Interrupt
    - [x] External interrupt
    - [ ] (WIP) Virt-io driver
* Process and Scheduling
    - [x] Switch to User-mode
    - [x] Process
    - [x] System call
    - [x] Scheduling
    - [x] Test multiple process scheduling
    - [x] Fork system call
    - [x] Timer-interrupt-based scheduling
    - [x] Multi-core support
    - [x] Use initcode instead of init binary
    - [ ] Allocator and stdlib in user-space
* Filesystem
    - [x] Fake fs and exec system call
    - [x] Real spinlock instead of nulllock
* Miscellaneous
    - [ ] Use Option instead of panic!
    - [ ] Eliminate use of unsafe
    - [ ] Documentation
    - [ ] High-level abstractions (driver, vm, etc.)
    - [ ] Port to aarch64 and deploy on Raspi
    - [x] Rewrite and credit code from other sources
    - [ ] Security issues

## Reference

[1] https://github.com/rust-embedded/rust-raspi3-OS-tutorials

[2] https://github.com/bztsrc/raspi3-tutorial/

[3] https://os.phil-opp.com/

[4] http://osblog.stephenmarz.com/

[5] https://github.com/mit-pdos/xv6-riscv/

[6] https://pdos.csail.mit.edu/6.828/2012/labs

[7] https://gist.github.com/cb372/5f6bf16ca0682541260ae52fc11ea3bb

## Known Issues

* Issue of `!`: function of `!` return type may interfere with RAII (objects won't be dropped). If function of return type `!` is called, there may be possible memory leak. Rust should drop all objects before calling these functions (instead of stack rewinding)
