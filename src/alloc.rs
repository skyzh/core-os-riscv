pub struct Allocator {
    pub lst_allocated: usize
}

extern "C" {
    static HEAP_START: usize;
    static HEAP_SIZE: usize;
}

pub const PAGE_SIZE: usize = 1 << 12;
const PAGE_ORDER: usize = 12;

pub const fn align_val(val: usize, order: usize) -> usize {
	let o = (1usize << order) - 1;
	(val + o) & !o
}

use crate::println;
impl Allocator {
    pub fn new() -> Self {
        Allocator {
            lst_allocated: align_val(unsafe { HEAP_START }, PAGE_ORDER)
        }
    }

    pub fn allocate(&mut self, size: usize) -> *mut u8 {
        unsafe {
            let acquired_page = self.lst_allocated;
            self.lst_allocated = align_val(self.lst_allocated + size, PAGE_ORDER);
            acquired_page as *mut u8
        }
    }
}

use lazy_static::lazy_static;
use crate::nulllock::Mutex;

lazy_static! {
    pub static ref ALLOCATOR: Mutex<Allocator> = Mutex::new(Allocator::new());
}

pub fn test() {
    unsafe {
        println!("{} {}", HEAP_START, HEAP_SIZE);
    }
}
