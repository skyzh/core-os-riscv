# core-os-riscv

An xv6-like operating system in Rust.

This project is originally based on "The Adventures of OS: Making a RISC-V Operating System using Rust",
and is now being made to have an xv6-like structure.

You may browse this repo with [Sourcegraph](https://sourcegraph.com/github.com/skyzh/core-os-riscv).

![image](https://user-images.githubusercontent.com/4198311/75318060-54def480-58a4-11ea-9051-604cb9dbae7f.png)

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

[![Build Status](https://travis-ci.com/skyzh/core-os-riscv.svg?branch=master)](https://travis-ci.com/skyzh/core-os-riscv)
<small>(documentation automatically built with travis)</small>

Documentation of this repo is automatically built and deployed with Travis. You may view online version 
[here](https://skyzh.github.io/core-os-riscv/kernel/), 
syscall specification [here](https://skyzh.github.io/core-os-riscv/user/syscall/index.html).

```bash
make && make docs
```

I'll continuously add Rust-specific implementations and how I made this project into documentation.

## Roadmap

**Update** I haven't maintained this project for a while, as I'm busy with my course-works. The long-term plan of this project is that, I'll leverage Rust async features to schedule kernel threads, therefore eliminating the need of sleeplock.

The main goal of this project is to make an xv6-like operating system with the Rust programming language.
And then, I'll separate arch-dependent part and make it into an OS supporting multiple architecture and
multiple boards.

- [x] Adapt code from http://osblog.stephenmarz.com/

* Virtual Memory and Management
    - [x] Virtual Memory
    - [x] Load ELF files from memory
    - [x] Kernel Allocator
    - [x] Remove direct call to allocator
    - [ ] (WIP) Add guard page around stack page
* Traps and Interrupt, Drivers
    - [x] UART drivers
    - [x] Machine-mode Timer Interrupt
    - [x] External interrupt
    - [x] Spinlock-based Virt-IO driver
    - [x] Sleeplock-based Virt-IO driver ([#2](https://github.com/skyzh/core-os-riscv/issues/2))
    - [ ] Handle signals in a Rust way ([#1](https://github.com/skyzh/core-os-riscv/issues/1))
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
    - [ ] (WIP) Implement wait syscall
    - [ ] Simple shell
    - [x] Investigate frequent kernel panic ([#8](https://github.com/skyzh/core-os-riscv/issues/8))
    - [ ] Reimplement process scheduling system ([#9](https://github.com/skyzh/core-os-riscv/issues/9))
* Filesystem
    - [x] Fake fs and exec system call
    - [x] Real spinlock instead of nulllock
    - [x] Implement simple fs ([#5](https://github.com/skyzh/core-os-riscv/issues/5))
    - [x] Implement read, write, open, close, dup, etc. syscalls
    - [x] Implement file-related syscalls on file system and eliminate use of Mutex ([#5](https://github.com/skyzh/core-os-riscv/issues/5))
    - [ ] Implement pipe
    - [ ] Copyin and Copyout implementation
    - [ ] Don't use Box in fs implementation
* Miscellaneous
    - [ ] (WIP) Replace Makefile with pure Rust toolchain (cargo build script)
    - [ ] Use Option instead of panic!
    - [ ] Eliminate use of unsafe
    - [ ] Documentation
    - [ ] High-level abstractions (driver, vm, etc.)
    - [ ] Port to aarch64 and deploy on Raspi
    - [x] Rewrite and credit code from other sources
    - [ ] RISC-V codegen issues ([#6](https://github.com/skyzh/core-os-riscv/issues/6))
    - [ ] Security issues ([#7](https://github.com/skyzh/core-os-riscv/issues/7))

## Reference

* https://github.com/rust-embedded/rust-raspi3-OS-tutorials
* https://github.com/bztsrc/raspi3-tutorial/
* https://os.phil-opp.com/
* http://osblog.stephenmarz.com/
* https://github.com/mit-pdos/xv6-riscv/
* https://pdos.csail.mit.edu/6.828/2012/labs
* https://gist.github.com/cb372/5f6bf16ca0682541260ae52fc11ea3bb
* https://github.com/rcore-os/rCore

For how these projects are related to core-os, refer to [#3](https://github.com/skyzh/core-os-riscv/issues/3).

## License

MIT