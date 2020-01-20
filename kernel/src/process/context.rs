#[repr(C)]
#[repr(align(4096))]
pub struct Context {

}

impl Context {
    pub const fn zero() -> Self {
        Self {}
    }
}
