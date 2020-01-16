pub const MAX_PAGE: usize = 128 * 1024 * 1024 / (1 << 12);

pub struct Allocator {
    pub base_addr: usize,
    pub page_allocated: [usize;MAX_PAGE]
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
            base_addr: align_val(unsafe { HEAP_START }, PAGE_ORDER),
            page_allocated: [0;MAX_PAGE]
        }
    }

    fn offset_addr_of(&self, id: usize) -> usize {
        let addr = self.base_addr + id * PAGE_SIZE;
        addr
    }
    unsafe fn offset_id_of(&self, id: usize) -> *mut u8 {
        self.offset_addr_of(id) as *mut u8
    }

    fn offset_page_of(&self, page: *mut u8) -> usize {
        let id = (page as usize - self.base_addr) / PAGE_SIZE;
        id
    }

    pub fn allocate(&mut self, size: usize) -> *mut u8 {
        let page_required = align_val(size, PAGE_ORDER) / PAGE_SIZE;
        for i in 0..MAX_PAGE {
            if self.page_allocated[i] == 0 {
                let mut found = true;
                for j in 0..page_required {
                    if self.page_allocated[i + j] != 0 {
                        found = false;
                        break;
                    }
                }
                if found {
                    for j in 0..page_required {
                        self.page_allocated[i + j] = page_required;
                    }
                    unsafe { return self.offset_id_of(i); }
                }
            }
        }
        panic!("no available page")
    }

    pub fn deallocate(&mut self, addr: *mut u8) {
        let id = self.offset_page_of(addr);
        let page_stride = self.page_allocated[id];
        for j in 0..page_stride {
            self.page_allocated[j + id] = 0;
        }
    }

    pub fn debug(&self) {
        let mut j = 0;
        loop {
            let size = self.page_allocated[j];
            if size != 0 {
                let from = self.offset_addr_of(j);
                let to = self.offset_addr_of(j + size);
                println!("{:X}-{:X} (pages: {})", from, to, size);
                j += size;
            } else {
                j += 1;
            }
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
