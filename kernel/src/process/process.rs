#[repr(C)]
#[derive(Clone, Copy)]
#[repr(align(4096))]
pub struct Process {

}

impl Process {
    pub const fn zero() -> Self {
        Self {}
    }
}