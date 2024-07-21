use crate::*;

extern "C" {
    static _heap_start: usize;
    static _heap_size: usize;
}

pub struct HeapAllocator {
    heap_start: usize,
    heap_size: usize,
    idx: spin::Mutex<usize>, // A simple pointer to last used page, we are going to change it ! Dono wori mai fryent
}
impl HeapAllocator {
    pub fn pages(&self, ) -> HeapPageIterator<'_> {
        HeapPageIterator::new(self)
    }
    pub fn alloc(&self, page_count: usize) -> Option<*mut Page> {
        assert!(page_count>0);
        assert!(self.heap_size>page_count);
        'pages: for (i,page_ptr) in self.pages().enumerate() {
            let page = unsafe {&*page_ptr}; // It's safe because we assume that our page iterator is yielding safe pages
            if page.is_zeroes() {
                let start_ptr = page_ptr;
                for j in i..i+page_count {
                    if unsafe{(*page_ptr.add(j)).is_taken()} {
                        continue 'pages;
                    }
                }
                *self.idx.lock() += page_count;
                return Some(start_ptr)
            }
        }
        None
    }
    //TODO Return Result
    pub fn dealloc(&self, mut start: *mut Page, page_count: usize) {
        let start_addr = start as usize;
        println!("{}", start_addr);
        todo!()
    }
    pub fn heap_end(&self) -> usize {self.heap_start+self.heap_size}
}

unsafe impl core::alloc::GlobalAlloc for HeapAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let ptr = Self::alloc(self, layout.size().div_ceil(4096)).unwrap();
        // let alignment = layout.align();
        // println!("{} {}", alignment, layout.size());
        ptr as _
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        // println!("WARNING: Deallocation is not implemented !");
    }
}

pub struct HeapPageIterator<'a> {
    heap_allocator: &'a HeapAllocator,
    curr: usize
}
impl<'a> HeapPageIterator<'a> {
    pub fn new(heap_alloc: &'a HeapAllocator) -> Self {
        Self {
            heap_allocator: heap_alloc,
            curr: 0,
        }
    }
}
impl<'a> core::iter::Iterator for HeapPageIterator<'a> {
    type Item = *mut Page;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr >= self.heap_allocator.heap_end() {return None}
        self.curr += 1;
        let base = *self.heap_allocator.idx.lock();
        Some(unsafe {(self.heap_allocator.heap_start as *mut Page).add(self.curr-1+base)})
    }
}
#[repr(C)]
pub struct Page {
    _raw: [u8; 4096]
}
impl Page {
    pub fn is_zeroes(&self) -> bool {
        self._raw == [0;4096]
    }
    pub fn is_taken(&self) -> bool {
        !self.is_zeroes()
    }
}

#[global_allocator]
pub static mut MAIN_HEAP_ALLOCATOR: HeapAllocator = HeapAllocator {heap_start:0,heap_size:0, idx: spin::Mutex::new(0), };

pub fn init() {
    info!("Initialising heap...");
    // Idk why, it should be the value, but the value is 0 and the address is the value...
    let heap_start = unsafe{core::ptr::addr_of!(_heap_start) as usize};
    let heap_size = unsafe{core::ptr::addr_of!(_heap_size) as usize};
    unsafe{MAIN_HEAP_ALLOCATOR.heap_start = heap_start-(heap_start%4096)+4096};
    unsafe{MAIN_HEAP_ALLOCATOR.heap_size = heap_size};
    assert!(heap_size>1_000_000, "Don't have enough memory !");
    assert_eq!(unsafe{MAIN_HEAP_ALLOCATOR.heap_start%4096}, 0);
    // We now have Vecs & others ! 
}

pub fn kalloc(page_count: usize) -> Option<*mut Page> {
    unsafe {
        MAIN_HEAP_ALLOCATOR.alloc(page_count)
    }
}