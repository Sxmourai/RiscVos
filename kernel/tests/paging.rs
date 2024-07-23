#![no_std]
#![feature(custom_inner_attributes)]
#![clippy::allow(all)]

pub fn paging_rw() {
    use crate::paging::*;
    let addr = crate::riscv::stack_start() as u64;
    map!(addr);
    let vaddr = Sv39VirtualAddress(addr);
    let paddr = Sv39PhysicalAddress(addr);
    assert_eq!(get_page(vaddr).unwrap().0, PageTableEntry::with_phys_pn(paddr).apply_flags(PageTableEntryFlags::rwx()).0);
    unsafe {core::ptr::write_volatile(vaddr.0 as *mut u8, 10)};
    assert_eq!(unsafe {core::ptr::read_volatile(vaddr.0 as *const u8)}, 10);
}