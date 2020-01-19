global_asm!(include_str!("usys.S"));

extern "C" {
    pub fn __write(fd: i32, content: &'static str, size: i32) -> i32;
}
