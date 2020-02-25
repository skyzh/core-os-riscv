// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

//! File in core-os including file in filesystem, device, pipe and symbol link

mod device;
pub use device::*;

mod fsfile;
pub use fsfile::*;

use alloc::boxed::Box;

/// File in core-os
pub enum File {
    Device(Box<dyn Device>),
    FsFile(FsFile),
    Pipe
}
