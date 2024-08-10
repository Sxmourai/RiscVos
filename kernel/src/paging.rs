use core::cell::OnceCell;

use bitfield::bitfield;
use riscv::{memory_start, stack_end, SATP};

use crate::*;

pub const PAGE_SIZE: usize = 4096; // 0x1000
pub const PAGE_SIZE64: u64 = 0x1000;

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
            PagingModes::Sv39 => 8<<60,
            PagingModes::Sv48 => 9<<60,
            PagingModes::Sv56 => 10<<60,
            PagingModes::Sv64 => 11<<60,
            PagingModes::Bare => 0<<60,
        }
    }
}



bitfield! {
    // Only 56 bits
    pub struct Sv39PhysicalAddress(u64);
    impl Debug;

    pub page_offset, set_page_offset: 11, 0;

    pub ppn0, set_ppn0: 20, 12;
    pub ppn1, set_ppn1: 29, 21;
    pub ppn2, set_ppn2: 55, 30;
}
impl Clone for Sv39PhysicalAddress {
    fn clone(&self) -> Self { *self }
}
impl Copy for Sv39PhysicalAddress {}
impl core::ops::Add<u64> for Sv39PhysicalAddress {
    type Output = Self;

    fn add(self, rhs: u64) -> Self::Output {
        Self(self.0+rhs)
    }
}

bitfield! {
    // Only 39 bits (Sv39... duh)
    pub struct Sv39VirtualAddress(u64);
    impl Debug;

    pub page_offset, set_page_offset: 11, 0;

    pub vpn0, set_vpn0: 20, 12;
    pub vpn1, set_vpn1: 29, 21;
    pub vpn2, set_vpn2: 38, 30;
}
impl Sv39VirtualAddress {
    pub fn new(addr: u64) -> Self {
        
        Self(addr)
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
impl core::ops::Add<u64> for Sv39VirtualAddress {
    type Output = Self;

    fn add(self, rhs: u64) -> Self::Output {
        Self(self.0+rhs)
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
impl PageTableEntryFlags {
    /// A **valid** entry with Read Write and Execute perms
    pub fn rwx() -> Self {
        Self(0b1111)
    }
    pub fn rwx_invalid() -> Self {
        Self(0b1110)
    }
}
bitfield! {
    pub struct PageTableEntry(u64);
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

    pub rsw, set_rsw: 9, 8;

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
        Sv39PhysicalAddress((self.0<<2)&!(0xFFF)) // Get only bits 56-10 and shift by 2 to right 
    }
    // If Read or Write or Execute bit is set, then it is a leaf, else it's a branch
    pub fn is_leaf(self) -> bool {
        self.0 & 0b1110 != 0 && self.valid()
    }
    pub fn apply_flags(self, flags: PageTableEntryFlags) -> Self {
        Self(self.0 | flags.0)
    }
    /// # Safety
    /// Caller must ensure that the entry is valid and the address is also valid
    pub unsafe fn get_page_table(self) -> &'static PageTable {
        let addr = self.parse_ppn();
        unsafe{&*(addr.0 as *mut _)}
    }
}
impl core::fmt::Debug for PageTableEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if !self.valid() {
            f.write_str("Invalid ")?;
        } else if self.is_leaf() {
            f.write_str("Leaf ")?;
        }
        f.write_str("Entry: ")?;
        if self.readable()   {core::fmt::Write::write_char(f, 'R')?}
        else {core::fmt::Write::write_char(f, '-')?}
        if self.writable()   {core::fmt::Write::write_char(f, 'W')?}
        else {core::fmt::Write::write_char(f, '-')?}
        if self.executable() {core::fmt::Write::write_char(f, 'X')?}
        else {core::fmt::Write::write_char(f, '-')?}
        if self.dirty()   {core::fmt::Write::write_char(f, 'D')?}
        if self.accessed() {core::fmt::Write::write_char(f, 'A')?}
        if self.user_mode_accessible()   {core::fmt::Write::write_char(f, 'U')?}
        if self.global_mapping() {core::fmt::Write::write_char(f, 'G')?}
        let ppn = self.parse_ppn();
        f.write_fmt(format_args!(" {} {} {}", ppn.ppn0(),ppn.ppn1(),ppn.ppn2()))?;
        Ok(())
    }
}
#[repr(C)]
pub struct PageTable {
    pub entries: [PageTableEntry; 512],
}

impl Default for PageTable {
    fn default() -> Self {
        Self::new()
    }
}

impl PageTable {
    pub fn new() -> Self {
        Self {
            entries: [PageTableEntry(0b0); 512],
        }
    }
    fn recurse_dbg(&self, tabs:usize, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for entry in self.entries {
            if entry.valid() {
                f.write_fmt(format_args!("{}{:?}", "\t".repeat(tabs), entry))?;
                if !entry.is_leaf() {
                    f.write_str(":\n")?;
                    unsafe{entry.get_page_table()}.recurse_dbg(tabs+1, f)?;
                }
                f.write_str("\n")?;
            }
        }
        Ok(())
    }
    /// # Safety
    /// Ultimate memory breaker
    pub unsafe fn map(&mut self, vaddr: Sv39VirtualAddress, paddr: Sv39PhysicalAddress, flags: PageTableEntryFlags) -> Result<&'_ mut PageTableEntry, PagingError> {
        assert!(PageTableEntry(flags.0).is_leaf());
        let mut level = 3;
        let mut current_page_table = self;
        let leaf_pte = loop {
            if level == 0 {
                panic!("Not found ?")
            }
            level -= 1;
            let pte = &mut current_page_table.entries[vaddr.vpn(level) as usize];
            // CAUTION! If accessing pte violates a PMA or PMP check, raise an access-fault exception corresponding to the
            // original access type
            if !pte.valid() {
                *pte = PageTableEntry(0b1); // Valid entry
                if level == 0 {break pte;}
                pte.0 |= PageTableEntry::with_phys_pn(Sv39PhysicalAddress(kalloc(1).unwrap() as u64)).0;
                trace!("Level: {}, Writing new entry at idx {}, pointing to: {:x}", level, vaddr.vpn(level), pte.parse_ppn().0);
            }
            else if !pte.readable() && pte.writable() {
                warn!("Invalid state entry !");
                return Err(PagingError::InvalidEntry(pte.clone()))
            }
            else if pte.is_leaf() {
                break pte;
            }
            current_page_table = unsafe{&mut *(pte.parse_ppn().0 as *mut PageTable)};
        };
        *leaf_pte = PageTableEntry::with_phys_pn(paddr).apply_flags(flags);
        Ok(leaf_pte)
    }
    /// # Safety
    /// Like map
    pub unsafe fn map_range(&mut self, start: Sv39VirtualAddress, page_count: u64, flags: PageTableEntryFlags) -> Result<(), PagingError> {
        for i in 0..page_count {
            let page_idx = i*PAGE_SIZE64;
            let page_addr = start.0+page_idx;
            unsafe{self.map(Sv39VirtualAddress(page_addr), Sv39PhysicalAddress(page_addr), flags)?};
        }
        Ok(())
    }
}
impl core::ops::Index<usize> for PageTable {
    type Output = PageTableEntry;
    fn index(&self, idx: usize) -> &Self::Output {
        &self.entries[idx]
    }
}
impl core::ops::IndexMut<usize> for PageTable {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        &mut self.entries[idx]
    }
}
impl core::fmt::Debug for PageTable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.recurse_dbg(0, f)
    }
}

pub fn get_root_pt() -> Result<&'static mut PageTable, PagingError> {
    let satp = SATP::read();
    if satp.mode() == 0 {
        // warn!("Trying to get page table but mode is BARE !");
        return Err(PagingError::InvalidPageTable);
    }
    let addr = satp.ppn()<<12;
    if addr == 0 {return Err(PagingError::InvalidPageTable);}
    Ok(unsafe{&mut *(addr as *mut _)})
}
/// Automatically gets root page table, and page-level
/// Not unsafe for convenience, just know that any call to this function is unsafe
/// Return: result
/// # Safety
/// Ultimate memory breaker, could write to different virtual addresses but be on same physical etc...

#[derive(Debug)]
pub enum PagingError {
    InvalidPageTable,
    InvalidEntry(PageTableEntry)
}

pub fn get_page(vaddr: Sv39VirtualAddress) -> Result<&'static PageTableEntry, PagingError> {
    let mut level = 3;
    let mut current_page_table = &*get_root_pt()?;
    let leaf_pte = loop {
        if level == 0 {
            panic!("Not found ?")
        }
        level -= 1;
        let pte = &current_page_table.entries[vaddr.vpn(level) as usize];
        if !pte.valid() {
            return Err(PagingError::InvalidEntry(pte.clone()));
        }
        else if !pte.readable() && pte.writable() {panic!("Invalid state entry !")}
        else if pte.is_leaf() {
            break pte;
        }
        current_page_table = unsafe{&*(pte.parse_ppn().0 as *const PageTable)};
    };
    Ok(leaf_pte)
}

#[macro_export]
macro_rules! map {
    ($vaddr: expr) => {
        $crate::map!($vaddr, $vaddr, flags=$crate::paging::PageTableEntryFlags::rwx())
    };
    ($vaddr: expr, count=$count: expr) => {
        {$crate::paging::get_root_pt()?.map_range($crate::paging::Sv39VirtualAddress($vaddr as _), $count, $crate::paging::PageTableEntryFlags::rwx())?}
    };
    ($vaddr: expr, $paddr: expr) => {
        $crate::map!($vaddr, $paddr, flags=$crate::paging::PageTableEntryFlags::rwx())
    };
    ($vaddr: expr, flags=$flags: expr) => {
        $crate::map!($vaddr, $vaddr, $flags) // Or $crate::heap::kalloc(1).unwrap() as *mut $crate::paging::PageTable
    };
    ($vaddr: expr, $paddr: expr, flags=$flags: expr) => {
        {$crate::paging::get_root_pt().unwrap().map($crate::paging::Sv39VirtualAddress($vaddr as _), $crate::paging::Sv39PhysicalAddress($paddr as _), $flags).unwrap()}
    };
}

pub fn init() {
    info!("Initialising paging...");// 80000ea0 3cf489c0 8081 0000  4bef18c0

    let mut satp = riscv::SATP(PagingModes::Sv39.satp());
    let root_page_table_ptr = crate::heap::kalloc(1).unwrap() as *mut PageTable;
    satp.set_ppn((root_page_table_ptr as u64) >> 12); // 2^12=4096
    let rpt = unsafe {&mut *(root_page_table_ptr)};
    for page in (memory_start()..stack_end()+PAGE_SIZE*200).step_by(PAGE_SIZE) {
        unsafe{rpt.map(Sv39VirtualAddress(page as _), Sv39PhysicalAddress(page as _), PageTableEntryFlags(0b1111)).unwrap()};
    }
    unsafe { csrw!("satp", satp.0) };
    // Map UART
    unsafe{map!(0x1000_0000)};
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
