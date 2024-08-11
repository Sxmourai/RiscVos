use paging::{get_root_pt, PageTableEntryFlags, PagingError, Sv39VirtualAddress, PAGE_SIZE};

use crate::*;
pub struct HeapAllocator {
    page_heap_start: usize,
    page_heap_size: usize,
    idx: spin::Mutex<usize>, // A simple pointer to last used page, we are going to change it ! Dono wori mai fryent
}
impl HeapAllocator {
    // pub fn pages(&self, ) -> HeapPageIterator<'_> {
    //     HeapPageIterator::new(self)
    // }
    pub fn idx(&self) -> usize {
        *self.idx.lock()
    }
    #[track_caller]
    pub fn alloc(&self, page_count: usize) -> Result<*mut Page, AllocationError> {
        assert!(page_count>0);
        let mut idx = self.idx.lock();
        let page_start = unsafe {((self.page_heap_start*PAGE_SIZE) as *mut Page).add(*idx)};
        if *idx+page_count>self.page_heap_size {return Err(AllocationError::NoPagesAvailable)}
        unsafe {
            if let Ok(rpt) = get_root_pt() {
                rpt.map_range(Sv39VirtualAddress(page_start as u64), page_count as u64, PageTableEntryFlags::rwx())?
            }
        }
        // log::debug!("Allocated {} base: {:?}", page_count, page_start);
        *idx += page_count;
        Ok(page_start)
        // 'pages: while *idx < self.page_heap_size {
        //     unsafe {
        //         if let Ok(rpt) = paging::get_root_pt() {
        //             rpt.map(Sv39VirtualAddress(page_ptr as _),
        //                     Sv39PhysicalAddress(page_ptr as _), PageTableEntryFlags::rwx()).or_else(|e| Err(AllocationError::Paging(e)))?;
        //         }
        //     }
        //     let page = unsafe {&*page_ptr}; // It's safe because we assume that our page iterator is yielding safe pages
        //     if page.is_zeroes() {
        //         let start_ptr = page_ptr;
        //         for j in i..i+page_count {
        //             if unsafe{(*page_ptr.add(j)).is_taken()} {
        //                 continue 'pages;
        //             }
        //         }
        //         *self.idx.lock() += page_count;
        //         return Ok(start_ptr)
        //     }
        //     unsafe {page_ptr = page_ptr.add(1)};
        // }
        // Err(AllocationError::NoPagesAvailable)
    }
    //TODO Return Result
    pub fn dealloc(&self, start: *mut Page, _page_count: usize) {
        let _start_addr = start as usize;
        // println!("{}", start_addr);
        // todo!()
    }
    pub fn page_heap_end(&self) -> usize {self.page_heap_start+self.page_heap_size}
}

pub struct LittleAllocator {
    start: usize,
    size: usize,
    current_offset: usize,
}
impl LittleAllocator {
    pub fn end(&self) -> usize {self.start+self.size}
    pub fn alloc(&mut self, size: usize) -> *mut u8 {
        assert!(self.start+size<=self.end());
        (self.start+self.current_offset) as _
    }
}


#[derive(Debug)]
pub enum AllocationError {
    Paging(PagingError),
    NoPagesAvailable,
}
impl From<PagingError> for AllocationError {
    fn from(value: PagingError) -> Self {
        Self::Paging(value)
    }
}

unsafe impl core::alloc::GlobalAlloc for HeapAllocator {
    #[track_caller]
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        if layout.size() <= 512 {
            return unsafe{LITTLE_HEAP_ALLOCATOR.alloc(layout.size())}
        }
        let ptr = Self::alloc(self, layout.size().div_ceil(PAGE_SIZE)).unwrap();
        let alignment = layout.align();
        println!("{} {}", alignment, layout.size());
        ptr as _
    }

    #[track_caller]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        Self::dealloc(self, ptr as _, layout.size().div_ceil(PAGE_SIZE))
    }
}

// pub struct HeapPageIterator<'a> {
//     heap_allocator: &'a HeapAllocator,
//     curr: usize
// }
// impl<'a> HeapPageIterator<'a> {
//     pub fn new(heap_alloc: &'a HeapAllocator) -> Self {
//         Self {
//             heap_allocator: heap_alloc,
//             curr: heap_alloc.idx(),
//         }
//     }
// }
// impl<'a> core::iter::Iterator for HeapPageIterator<'a> {
//     type Item = *mut Page;

//     fn next(&mut self) -> Option<Self::Item> {
//         if self.curr >= self.heap_allocator.page_heap_end() {return None}
//         self.curr += 1;
//         let base = *self.heap_allocator.idx.lock();
//         Some(unsafe {(self.heap_allocator.heap_start as *mut Page).add(self.curr-1+base)})
//     }
// }
#[repr(C)]
pub struct Page {
    _raw: [u8; 4096]
}
impl Page {
    // pub fn is_zeroes(&self) -> bool {
    //     self._raw == [0;4096]
    // }
    // pub fn is_taken(&self) -> bool {
    //     !self.is_zeroes()
    // }
}

#[global_allocator]
pub static mut MAIN_HEAP_ALLOCATOR: HeapAllocator = HeapAllocator {page_heap_start:0,page_heap_size:0, idx: spin::Mutex::new(0), };
pub static mut LITTLE_HEAP_ALLOCATOR: LittleAllocator = LittleAllocator {start:0,size:0, current_offset: 0, };

pub fn init() {
    info!("Initialising heap...");
    // Idk why, it should be the value, but the value is 0 and the address is the value...
    unsafe{MAIN_HEAP_ALLOCATOR.page_heap_start = heap_start().div_ceil(PAGE_SIZE)};
    unsafe{MAIN_HEAP_ALLOCATOR.page_heap_size = heap_size()/PAGE_SIZE};
    unsafe{LITTLE_HEAP_ALLOCATOR.start = heap_start()+10*1024};
    unsafe{LITTLE_HEAP_ALLOCATOR.size = 8*1024};
    assert!(heap_size()>5000, "Don't have enough memory !");
    // assert_eq!(unsafe{MAIN_HEAP_ALLOCATOR.page_heap_start}, 0);
    // We now have Vecs & others ! 
}

pub fn kalloc(page_count: usize) -> Result<*mut Page, AllocationError> {
    unsafe {
        MAIN_HEAP_ALLOCATOR.alloc(page_count)
    }
}
pub fn kmalloc(bytes: usize) -> Result<*mut Page, AllocationError> {
    unsafe {
        Ok(LITTLE_HEAP_ALLOCATOR.alloc(bytes) as _)
    }
}