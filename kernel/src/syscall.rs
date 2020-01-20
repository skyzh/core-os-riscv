use crate::process::{TrapFrame, Register};
use crate::trap::my_proc;
use crate::info;
pub use crate::syscall_gen::*;

pub fn argraw(tf: &TrapFrame, pos: usize) -> usize {
    match pos {
        0 => tf.regs[Register::a0 as usize],
        1 => tf.regs[Register::a1 as usize],
        2 => tf.regs[Register::a2 as usize],
        3 => tf.regs[Register::a3 as usize],
        4 => tf.regs[Register::a4 as usize],
        5 => tf.regs[Register::a5 as usize],
        _ => unreachable!()
    }
}

pub fn argint(tf: &TrapFrame, pos: usize) -> i64 {
    argraw(tf, pos) as i64
}

fn sys_write(fd: i32, content: u64, sz: i32) {}

pub fn syscall() {
    let syscall_id;
    {
        let p = my_proc().lock();
        let tf = &p.trapframe;
        syscall_id = tf.regs[Register::a7 as usize] as i64;
    }
    match syscall_id {
        SYS_WRITE => sys_write(0, 0, 0),
        _ => unreachable!()
    }
}
