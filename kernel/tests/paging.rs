#![no_std]
#![feature(custom_inner_attributes)]
#![clippy::allow(all)]

pub fn manipulations() -> bool {
    let mut pte = kernel::paging::PageTableEntry(0);
    pte.set_ppn0(0x8000013/4096);
    pte.set_ppn0(0x1234/4096);
    pte.set_ppn0(0x5678/4096);
    kernel::dbg!(kernel::paging::PageTableEntry::with_phys_pn(pte.parse_ppn()).parse_ppn());
    kernel::dbg!(pte.parse_ppn());
    true
}