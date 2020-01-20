use crate::process::{TrapFrame, Register};
use crate::trap::my_proc;
use crate::{info, panic, print, println};
use crate::page;
pub use crate::syscall_gen::*;
use crate::alloc::{align_val, page_down};
use crate::symbols::{PAGE_ORDER, PAGE_SIZE};

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

pub fn argint(tf: &TrapFrame, pos: usize) -> i32 {
    argraw(tf, pos) as i32
}
pub fn arguint(tf: &TrapFrame, pos: usize) -> usize {
    let sz = argraw(tf, pos) as i32;
    if sz < 0 {
        panic!("invalid unsigned int");
    }
    sz as usize
}

pub fn argptr(pgtable: &page::Table, tf: &TrapFrame, pos: usize, sz: usize) -> *const u8 {
    let ptr = argraw(tf, pos);
    let pg_begin = page_down(ptr);
    if ptr + sz >= pg_begin + PAGE_SIZE {
        panic!("out of bound!");
    }
    let paddr = pgtable.paddr_of(pg_begin).unwrap();
    unsafe { (paddr as *const u8).add(ptr - pg_begin) }
}

fn sys_write() {
    let fd;
    let content;
    let sz;
    {
        let p = my_proc().lock();
        fd = argint(&p.trapframe, 0);
        sz = arguint(&p.trapframe, 2);
        content = argptr(&p.pgtable, &p.trapframe, 1, sz);
        println!("fd={}, sz={}, content=0x{:x}", fd, sz, content as usize);
    }
    for i in 0..sz {
        print!("{}", unsafe { *content.add(i) } as char);
    }
}

pub fn syscall() {
    let syscall_id;
    {
        let p = my_proc().lock();
        let tf = &p.trapframe;
        syscall_id = tf.regs[Register::a7 as usize] as i64;
    }
    match syscall_id {
        SYS_WRITE => sys_write(),
        _ => unreachable!()
    }
}
