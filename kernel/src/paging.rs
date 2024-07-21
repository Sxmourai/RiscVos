use core::cell::OnceCell;

use bitfield::bitfield;
use riscv::{enter_mode, get_mode, PrivilegeLevel, SATP};

use crate::*;

// PMP -> Physical memory protection
// PMP checks are only in Supervised or User mode
//////////// Or data accesses in Machine mode (our mode for now) when the MPRV bit is set
pub static mut MEMORY_PROTECTION: OnceCell<PMP> = OnceCell::new();

// Memory Management Unit
pub struct PMP {
    page_tables: [Table; 512],
}
impl Default for PMP {
    fn default() -> Self {
        Self::new()
    }
}

impl PMP {
    pub fn new() -> Self {
        Self {
            page_tables: core::array::from_fn(|_| Table::new()),
        }
    }
}
#[derive(Clone)]
pub struct Table {}
impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}

impl Table {
    pub fn new() -> Self {
        Self {}
    }
}
pub enum PagingModes {
    Sv32, // Only RV32
    Sv39, // Only supported
    Sv48,
    Sv56,
    Sv64, // Not totally defined in spec
    Bare, // no additional memory protection beyond the physical memory protection scheme
}
impl PagingModes {
    // Only RV64 for now
    pub fn satp(self) -> u64 {
        match self {
            PagingModes::Sv32 => todo!(),
            PagingModes::Sv39 => 8,
            PagingModes::Sv48 => 9,
            PagingModes::Sv56 => 10,
            PagingModes::Sv64 => 11,
            PagingModes::Bare => 0,
        }
    }
}


// pub static mut PageTables


bitfield! {
    pub struct PageTableEntry(u64);
    impl Debug;
    pub valid, set_valid: 0;
    pub readable, set_readable: 1;
    pub writable, set_writable: 2;
    pub executable, set_executable: 3;
    pub user_mode_accessible, set_user_mode_accessible: 4;
    // Global mappings are those that exist in all address spaces. For non-leaf PTEs, the global setting implies that all mappings in the subsequent levels of the page table are global.
    // Note that failing to mark a global mapping as global merely reduces performance, whereas
    // marking a non-global mapping as global is a software bug that, after switching to an address space
    // with a different non-global mapping for that address range, can unpredictably result in either mapping being used
    pub global_mapping, set_global_mapping: 5;
    pub accessed, set_accessed: 6;
    pub dirty, set_dirty: 7;

    pub rsw, set_rsw: 10, 8;

    pub ppn0, set_ppn0: 19, 10;
    pub ppn1, set_ppn1: 28, 19;
    pub ppn2, set_ppn2: 54, 28;

    reserved, _: 60, 53;

    // Chapter 12. "Svpbmt" Extension for Page-Based Memory Types, Version 1.0
    // Mode -Value - Requested Memory Attributes
    // PMA - 0 - None
    // NC  - 1 - Non-cacheable, idempotent, weakly-ordered (RVWMO), main memory
    // IO  - 2 - Non-cacheable, non-idempotent, strongly-ordered (I/O ordering), I/O
    pub pbmt, set_pbmt: 62, 60;

    // Chapter 11. SvNAPOT extension
    // If not available, should be set to 0 or else page-fault
    pub n, set_n: 62;
}
impl PageTableEntry {
    pub fn with_phys_pn(ppn: Sv39PhysicalAddress) -> Self {
        Self((ppn.0 & !(0xFFF)) >> 2)
    }
    pub fn parse_ppn(self) -> Sv39PhysicalAddress {
        Sv39PhysicalAddress((self.0&!(0x3FF))<<2)// Get only bits 56-10 and shift by 2 to right 
    }
    // If Read or Write or Execute bit is set, then it is a leaf, else it's a branch
    pub fn is_leaf(self) -> bool {
        self.0 & 0b1110 != 0 && self.valid()
    }
    pub fn apply_flags(self, flags: PageTableEntryFlags) -> Self {
        Self(self.0 | flags.0)
    }
}
// Can't derive cuz using bitfield
impl Clone for PageTableEntry {
    fn clone(&self) -> Self { *self }
}
impl Copy for PageTableEntry {}

bitfield! {
    pub struct PageTableEntryFlags(u64);
    impl Debug;
    pub valid, set_valid: 0;
    pub readable, set_readable: 1;
    pub writable, set_writable: 2;
    pub executable, set_executable: 3;
    pub user_mode_accessible, set_user_mode_accessible: 4;
    pub global_mapping, set_global_mapping: 5;
    pub accessed, set_accessed: 6;
    pub dirty, set_dirty: 7;

    pub rsw, set_rsw: 10, 8;

    pub pbmt, set_pbmt: 62, 60;
    pub n, set_n: 62;
}
impl Clone for PageTableEntryFlags {
    fn clone(&self) -> Self { *self }
}
impl Copy for PageTableEntryFlags {}

bitfield! {
    // Only 56 bits
    pub struct Sv39PhysicalAddress(u64);
    impl Debug;

    pub page_offset, set_page_offset: 11, 0;

    pub ppn0, set_ppn0: 21, 12;
    pub ppn1, set_ppn1: 30, 21;
    pub ppn2, set_ppn2: 56, 30;
}
impl Sv39PhysicalAddress {
    pub fn new(paddr: u64) -> Self {        
        let mut _s = Self(0);
        let frame = paddr/4096;
        _s.set_ppn0((frame)&0x1FF);
        _s.set_ppn1((frame>>9)&0x1FF);
        _s.set_ppn2((frame>>18)&0x3FF_FFFF);
        _s
    }
}
impl Clone for Sv39PhysicalAddress {
    fn clone(&self) -> Self { *self }
}
impl Copy for Sv39PhysicalAddress {}

bitfield! {
    // Only 39 bits (Sv39... duh)
    pub struct Sv39VirtualAddress(u64);
    impl Debug;

    pub page_offset, set_page_offset: 11, 0;

    pub vpn0, set_vpn0: 21, 12;
    pub vpn1, set_vpn1: 30, 21;
    pub vpn2, set_vpn2: 39, 30;
}
impl Sv39VirtualAddress {
    pub fn new(addr: u64) -> Self {
        let mut _s = Self(addr);
        // let page = addr/4096;
        // _s.set_vpn0((page)&0x1FF);
        // _s.set_vpn1((page>>9)&0x1FF);
        // _s.set_vpn2((page>>18)&0x1FF);
        // _s.set_page_offset(addr&0xFFF);
        _s
    }
    pub fn vpn(self, vpni: u64) -> u64 {
        assert!(vpni <= 3);
        (self.0 >> ((9 * vpni)+12)) & 0x1FF
    }
}
impl Clone for Sv39VirtualAddress {
    fn clone(&self) -> Self { *self }
}
impl Copy for Sv39VirtualAddress {}

pub struct PageTable {
    pub entries: [PageTableEntry; 512],
}

impl PageTable {
    pub fn new() -> Self {
        Self {
            entries: [PageTableEntry(0b0); 512],
        }
    }
    pub unsafe fn map(&mut self, vaddr: Sv39VirtualAddress, paddr: Sv39PhysicalAddress, flags: PageTableEntryFlags) {
        assert!(PageTableEntry(flags.0).is_leaf());
        dbg!(vaddr);
        let mut level = 3;
        assert!((get_mode() == PrivilegeLevel::Supervisor || get_mode() == PrivilegeLevel::User));

        let mut current_page_table = self;
        let leaf_pte = loop {
            // dbg!(Sv39PhysicalAddress(current_pt_addr));
            if level == 0 {
                panic!("Not found ?")
            }
            level -= 1;
            println!("\t----{}------", level);
            let mut pte = &mut current_page_table.entries[vaddr.vpn(level) as usize];
            // CAUTION! If accessing pte violates a PMA or PMP check, raise an access-fault exception corresponding to the
            // original access type
            if !pte.valid() {
                println!("\tWriting new entry at idx {}", vaddr.vpn(level));
                *pte = PageTableEntry(kalloc(1).unwrap() as u64>>2);
                pte.set_valid(true);
                if level == 0 {break pte;}
            }
            else if !pte.readable() && pte.writable() {panic!("Invalid state entry !")}
            else if pte.is_leaf() {
                break pte;
            }
            // dbg!(pte, current_pt_addr);
            current_page_table = match level-1 {
                0 => unsafe{&mut *(pte.parse_ppn().0 as *mut PageTable)},
                1 => unsafe{&mut *(pte.parse_ppn().0 as *mut PageTable)},
                2 => unsafe{&mut *(pte.parse_ppn().0 as *mut PageTable)},
                _ => {todo!()},
            };
            // pte.ppn(level).0*4096;
        };
        *leaf_pte = PageTableEntry::with_phys_pn(paddr).apply_flags(flags);
        dbg!(leaf_pte, leaf_pte.parse_ppn().0, paddr);
    }

    pub unsafe fn get_page(&self, vaddr: Sv39VirtualAddress) -> Option<&PageTableEntry> {
        let mut level = 3;
        assert!((get_mode() == PrivilegeLevel::Supervisor || get_mode() == PrivilegeLevel::User));

        let mut current_page_table = self;
        let leaf_pte = loop {
            dbg!(level);
            level -= 1;
            if level == 0 {
                dbg!("Not found ?");
                return None;
            }
            let mut pte = &current_page_table.entries[vaddr.vpn(level) as usize];
            if !pte.valid() {
                return None;
            }
            else if !pte.readable() && pte.writable() {panic!("Invalid state entry !")}
            else if pte.is_leaf() {
                break pte;
            }
            current_page_table = match level-1 {
                0 => unsafe{&mut *(pte.parse_ppn().0 as *mut PageTable)},
                1 => unsafe{&mut *(pte.parse_ppn().0 as *mut PageTable)},
                2 => unsafe{&mut *(pte.parse_ppn().0 as *mut PageTable)},
                _ => {todo!()},
            };
            dbg!(core::ptr::addr_of!(current_page_table));
        };
        Some(leaf_pte)
    }
}

/// Automatically gets root page table, and page-level
/// Return: result
/// # Safety
/// Ultimate memory breaker, could write to different virtual addresses but be on same physical etc...
pub unsafe fn map(vaddr: Sv39VirtualAddress, paddr: Sv39PhysicalAddress, flags: PageTableEntryFlags) {
    assert!(PageTableEntry(flags.0).is_leaf());
    dbg!(vaddr);
    let mut current_pt_addr = unsafe {ROOT_PAGE_TABLE.unwrap()} as u64;
    let mut level = 3;
    assert!((get_mode() == PrivilegeLevel::Supervisor || get_mode() == PrivilegeLevel::User));

    let leaf_pte = loop {
        dbg!(Sv39PhysicalAddress(current_pt_addr));
        if level == 0 {
            panic!("Not found ?")
        }
        level -= 1;
        println!("\t----{}------", level);
        let pt = unsafe {&mut *(current_pt_addr as *mut PageTable)};
        let mut pte = &mut pt.entries[vaddr.vpn(level) as usize];
        // CAUTION! If accessing pte violates a PMA or PMP check, raise an access-fault exception corresponding to the
        // original access type
        if !pte.valid() {
            println!("\tWriting new entry at idx {}", vaddr.vpn(level));
            *pte = if level == 0 {PageTableEntry::with_phys_pn(paddr)} else {PageTableEntry(kalloc(1).unwrap() as u64>>2)};
            pte.set_valid(true);
            if level == 0 {
                break pte;
            }
        }
        else if !pte.readable() && pte.writable() {panic!("Invalid state entry !")}
        else if pte.is_leaf() {
            break pte;
        }
        dbg!(pte, current_pt_addr);
        current_pt_addr = match level-1 {
            0 => {pte.parse_ppn().0},
            1 => {pte.parse_ppn().0},
            2 => {pte.parse_ppn().0},
            _ => {todo!()},
        };
        // pte.ppn(level).0*4096;
    };
    assert_eq!(leaf_pte.parse_ppn().0, paddr.0);
    dbg!(leaf_pte, leaf_pte.parse_ppn().0, paddr);
    
}

static mut ROOT_PAGE_TABLE: Option<*mut PageTable> = None;


// This is from 10.3.2. Virtual Address Translation Process
/// Returns
/// If find the entry and is valid, returns the corresponding PageTableEntry inside Ok()
/// If entry is invalid, returns a ptr to the entry, inside Err()
pub fn get_page(va: Sv39VirtualAddress) -> Result<&'static mut PageTableEntry, *mut PageTableEntry> {
    // Step 1.
    dbg!(va);
    let satp = SATP(csrr!("satp"));
    let mut current_pt_addr = satp.ppn() * 4096;
    let mut level = 3;
    assert!((get_mode() == PrivilegeLevel::Supervisor || get_mode() == PrivilegeLevel::User));

    let leaf_pte = loop {
        dbg!(level, Sv39PhysicalAddress(current_pt_addr));
        if level == 0 {
            panic!("Not found ?")
        }
        level -= 1;
        // Step 2.
        let pt = unsafe {&mut *(current_pt_addr as *mut PageTable)};
        let mut pte = &mut pt.entries[(va.vpn(level) * (core::mem::size_of::<Sv39VirtualAddress>() as u64)) as usize];
        // CAUTION! If accessing pte violates a PMA or PMP check, raise an access-fault exception corresponding to the
        // original access type
        // let pte = unsafe { *(pte_addr as *mut PageTableEntry) };
        // Step 3.
        if !pte.valid() || (!pte.readable() && pte.writable()) {
            // || pte.any_reserved_combination
            // panic!("Page fault: trying to load from invalid entry !");
            let mut _pte = *pte; // Little cheat =)
            dbg_bits!(_pte.0);
            dbg!(pte);
            return Err(core::ptr::addr_of_mut!(_pte))
        }
        // Step 4.
        if pte.is_leaf() {
            // Step 5.
            dbg!(pte);
            break pte;
        }
        dbg!(pte);
        current_pt_addr = match level {
            0 => {pte.ppn0()},
            1 => {pte.ppn1()},
            2 => {pte.ppn2()},
            _ => {todo!()},
        }*4096;
        // pte.ppn(level).0*4096;
    };
    Ok(leaf_pte)
}

pub fn init() {
    // return;
    crate::println!("Initialising paging...");
    // unsafe{dbg!(csrr!("mstatus"))};
    unsafe { enter_mode(PrivilegeLevel::Supervisor) };
    // unsafe{dbg!(csrr!("mstatus"))};
    let mut satp = riscv::SATP(0);
    satp.set_mode(PagingModes::Sv39.satp());
    // satp.set_asid();
    let root_page_table_ptr = crate::heap::kalloc(1).unwrap() as *mut PageTable;
    let mut root_page_table = PageTable::new();
    // for page_table in &mut root_page_table.entries {
    //     page_table.set_readable(1);
    //     page_table.set_writable(1);
    //     page_table.set_executable(1);
    // }
    unsafe { *(root_page_table_ptr) = root_page_table };
    unsafe { ROOT_PAGE_TABLE.replace(root_page_table_ptr) };
    satp.set_ppn((root_page_table_ptr as u64) >> 12); // 2^12=4096
    unsafe { csrw!("satp", satp.0) };
    let vaddr = Sv39VirtualAddress(crate::traps::save_context as _);
    let paddr = Sv39PhysicalAddress(kalloc(1).unwrap() as u64);
    let mut flags = PageTableEntryFlags(0b1111); // XWRV
    unsafe{(*ROOT_PAGE_TABLE.unwrap()).map(vaddr, paddr, flags)}
    unsafe{(*ROOT_PAGE_TABLE.unwrap()).get_page(vaddr)};
    // let pte = get_page(vaddr);
    // crate::dbg!(pte);
    // crate::dbg_bits!(pte.unwrap().0);
    // unsafe{core::arch::asm!("csrw satp, {}", in(reg) satp)};
}


// Sv32:

// bitfield! {
//     pub struct PageTableEntry(u32);
//     impl Debug;
//     pub valid, set_valid: 1, 0;
//     pub readable, set_readable: 2, 1;
//     pub writable, set_writable: 3, 2;
//     pub executable, set_executable: 4, 3;
//     pub user_mode_accessible, set_user_mode_accessible: 5, 4;
//     // Global mappings are those that exist in all address spaces. For non-leaf PTEs, the global setting implies that all mappings in the subsequent levels of the page table are global.
//     // Note that failing to mark a global mapping as global merely reduces performance, whereas
//     // marking a non-global mapping as global is a software bug that, after switching to an address space
//     // with a different non-global mapping for that address range, can unpredictably result in either mapping being used
//     pub global_mapping, set_global_mapping: 6, 5;
//     pub accessed, set_accessed: 7, 6;
//     pub dirty, set_dirty: 8, 7;
// pub ppn0, set_ppn0: 20, 10;
// pub ppn1, set_ppn1: 32, 20;
// }
