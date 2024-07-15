use core::cell::OnceCell;

use bitfield::bitfield;

use crate::{csrr, csrw, dbg, dbg_bits, heap::kalloc};

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

pub enum CSRFieldSpec {
    WPRI, // 2.3.1. Reserved Writes Preserve Values, Reads Ignore Values (WPRI)
    WLRL, // 2.3.2. Write/Read Only Legal Values (WLRL)
    WARL, // 2.3.3. Write Any Values, Reads Legal Values (WARL)
}

bitfield::bitfield! {
    pub struct SATP(u64);
    impl Debug;
    pub mode, set_mode: 63, 60;
    pub asid, set_asid: 60, 44;
    pub ppn, set_ppn: 44, 0;
}
bitfield::bitfield! {
    pub struct MSTATUS(u64);
    impl Debug;

    pub sd,  set_sd: 63, 62;

    // Controls endianness (User, Machine, Supervisor)
    pub ube,  set_ube: 7, 6;
    pub mbe,  set_mbe: 38, 37;
    pub sbe,  set_sbe: 37, 36;
    // SXL and UXL control value of XLEN for S and U mode
    // The encoding of these fields is the same as the MXL field of misa, shown in
    // Table 9. The effective XLEN in S-mode and U-mode are termed SXLEN and UXLEN, respectively.
    // Not our case but: When MXLEN=32, the SXL and UXL fields do not exist, and SXLEN=32 and UXLEN=32.
    // the set of legal values that the UXL field may assume excludes those that
    // would cause UXLEN > SXLEN (== is fine)
    pub sxl, set_sxl: 36, 34; // 2 bits !
    pub uxl, set_uxl: 34, 32; // 2 bits !

    // Trap SRET
    pub tsr,  set_tsr: 23, 22;
    // (Timeout Wait)
    pub tw,   set_tw: 22, 21;
    // Trap Virtual Memory, used for virtualisation
    pub tvm,  set_tvm: 21, 20;
    // The MXR (Make eXecutable Readable) bit modifies the privilege with which loads access virtual
    // memory. When MXR=0, only loads from pages marked readable (R=1 in Figure 59) will succeed.
    // When MXR=1, loads from pages marked either readable or executable (R=1 or X=1) will succeed. MXR
    // has no effect when page-based virtual memory is not in effect. MXR is read-only 0 if S-mode is not
    // supported
    pub mxr,  set_mxr: 20, 19;
    // permit Supervisor User Memory access bit modifies the privilege with which S-mode loads
    // and stores access virtual memory. When SUM=0, S-mode memory accesses to pages that are
    // accessible by U-mode (U=1 in Figure 59) will fault. When SUM=1, these accesses are permitted. SUM
    // has no effect when page-based virtual memory is not in effect. Note that, while SUM is ordinarily
    // ignored when not executing in S-mode, it is in effect when MPRV=1 and MPP=S. SUM is read-only 0 if
    // S-mode is not supported or if satp.MODE is read-only 0.
    pub sum,  set_sum: 19, 18;
    // 3.1.6.3. Memory Privilege in mstatus Register: Modify privilege
    //  When MPRV=0, loads and stores behave as normal, using the translation and
    // protection mechanisms of the current privilege mode. When MPRV=1, load and store memory
    // addresses are translated and protected, and endianness is applied, as though the current privilege
    // mode were set to MPP. Instruction address-translation and protection are unaffected by the setting of
    // MPRV. MPRV is read-only 0 if U-mode is not supported.
    // An MRET or SRET instruction that changes the privilege mode to a mode less privileged than M also
    // sets MPRV=0.
    pub mprv, set_mprv: 18, 17;
    //todo Interesting for speeding up context switches
    pub xs,   set_xs: 17, 5; // 2 bits !
    pub fs,   set_fs: 15, 13; // 2 bits !
    //  When a trap is taken from privilege mode y
    // into privilege mode x, xPIE is set to the value of xIE; xIE is set to 0; and xPP is set to y
    pub mpp,  set_mpp: 13, 11; // 2 bits !, holds previous privilege mode for machine traps
    // M-mode software can determine whether a privilege mode is implemented by writing that
    // mode to MPP then reading it back.

    pub vs,   set_vs: 11, 9; // 2 bits !
    pub spp,  set_spp: 9, 8; // holds previous privilege mode for supervisor traps (cuz it can only be User, only 1 bit)
    pub mpie, set_mpie: 8, 7;
    pub spie, set_spie: 6, 5;
    pub mie,  set_mie: 4, 3; // Global interrupt-enable bits supervisor mode
    pub sie,  set_sie: 2, 1; // Global interrupt-enable bits machine mode
}

// pub static mut PageTables
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PrivilegeLevel {
    User = 0,       // Abbr: U
    Supervisor = 1, // Abbr: S
    Reserved = 2,   // Abbr:
    Machine = 3,    // Abbr: M
}
/// # Safety
/// Can break memory in a different ways, e.g. switching Machine mode with no paging and Supervisor mode
pub unsafe fn enter_mode(priv_level: PrivilegeLevel) {
    let mstatus = csrr!("mstatus");
    csrw!(
        "mstatus",
        mstatus & !(0b11 << 11) | (priv_level as u64) << 11
    )
}
pub fn get_mode() -> PrivilegeLevel {
    match MSTATUS(csrr!("mstatus")).mpp() {
        0 => PrivilegeLevel::User,
        1 => PrivilegeLevel::Supervisor,
        3 => PrivilegeLevel::Machine,
        _ => {
            todo!()
        } // Should never happen
    }
}
bitfield! {
    pub struct PageTableEntry(u64);
    impl Debug;
    pub valid, set_valid: 1, 0;
    pub readable, set_readable: 2, 1;
    pub writable, set_writable: 3, 2;
    pub executable, set_executable: 4, 3;
    pub user_mode_accessible, set_user_mode_accessible: 5, 4;
    // Global mappings are those that exist in all address spaces. For non-leaf PTEs, the global setting implies that all mappings in the subsequent levels of the page table are global.
    // Note that failing to mark a global mapping as global merely reduces performance, whereas
    // marking a non-global mapping as global is a software bug that, after switching to an address space
    // with a different non-global mapping for that address range, can unpredictably result in either mapping being used
    pub global_mapping, set_global_mapping: 6, 5;
    pub accessed, set_accessed: 7, 6;
    pub dirty, set_dirty: 8, 7;

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
    pub n, set_n: 63, 62;
}
impl PageTableEntry {
    pub fn is_valid(self) -> bool {
        self.valid() == 1
    }
    pub fn is_readable(self) -> bool {
        self.readable() == 1
    }
    pub fn is_writable(self) -> bool {
        self.writable() == 1
    }
    pub fn is_executable(self) -> bool {
        self.executable() == 1
    }
    pub fn is_user_mode_accessible(self) -> bool {
        self.user_mode_accessible() == 1
    }
    pub fn is_global_mapping(self) -> bool {
        self.global_mapping() == 1
    }
    pub fn is_accessed(self) -> bool {
        self.accessed() == 1
    }
    pub fn is_dirty(self) -> bool {
        self.dirty() == 1
    }
    pub fn with_phys_pn(ppn: Sv39PhysicalAddress) -> Self {
        Self((ppn.0 & !(0xFFF)) >> 2)
    }
    pub fn parse_ppn(self) -> Sv39PhysicalAddress {
        Sv39PhysicalAddress((self.0&!(0x3FF))<<2)// Get only bits 56-10 and shift by 2 to right 
    }
    // If Read or Write or Execute bit is set, then it is a leaf, else it's a branch
    pub fn is_leaf(self) -> bool {
        self.0 & 0b1110 != 0
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
    pub valid, set_valid: 1, 0;
    pub readable, set_readable: 2, 1;
    pub writable, set_writable: 3, 2;
    pub executable, set_executable: 4, 3;
    pub user_mode_accessible, set_user_mode_accessible: 5, 4;
    pub global_mapping, set_global_mapping: 6, 5;
    pub accessed, set_accessed: 7, 6;
    pub dirty, set_dirty: 8, 7;

    pub rsw, set_rsw: 10, 8;

    pub pbmt, set_pbmt: 62, 60;
    pub n, set_n: 63, 62;
}
impl Clone for PageTableEntryFlags {
    fn clone(&self) -> Self { *self }
}
impl Copy for PageTableEntryFlags {}

bitfield! {
    // Only 56 bits
    pub struct Sv39PhysicalAddress(u64);
    impl Debug;

    pub page_offset, set_page_offset: 12, 0;

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

bitfield! {
    // Only 39 bits (Sv39... duh)
    pub struct Sv39VirtualAddress(u64);
    impl Debug;

    pub page_offset, set_page_offset: 12, 0;

    pub vpn0, set_vpn0: 21, 12;
    pub vpn1, set_vpn1: 30, 21;
    pub vpn2, set_vpn2: 39, 30;
}
impl Sv39VirtualAddress {
    pub fn new(addr: u64) -> Self {
        let mut _s = Self(0);
        let page = addr/4096;
        _s.set_vpn0((page)&0x1FF);
        _s.set_vpn1((page>>9)&0x1FF);
        _s.set_vpn2((page>>18)&0x1FF);
        _s
    }
    pub fn vpn(self, vpni: u64) -> u64 {
        assert!(vpni <= 3);
        (self.0 >> (9 * vpni)) & 0x1FF
    }
}
impl Clone for Sv39VirtualAddress {
    fn clone(&self) -> Self { *self }
}
impl Copy for Sv39VirtualAddress {}

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
            entries: [PageTableEntry(0b1); 512],
        }
    }
}

/// Automatically gets root page table, and page-level
/// Return: result
/// # Safety
/// Ultimate memory breaker, could write to different virtual addresses but be on same physical etc...
pub unsafe fn map(vaddr: Sv39VirtualAddress, paddr: Sv39PhysicalAddress, flags: PageTableEntryFlags) {
    assert!(PageTableEntry(flags.0).is_leaf());
    match get_page(vaddr) {
        Ok(mut entry) => {*entry = entry.apply_flags(flags);},
        Err(mut entry) => {
            dbg!(entry);
            let mut pte = PageTableEntry(flags.0);
            pte.set_ppn0(paddr.ppn0());
            pte.set_ppn1(paddr.ppn1());
            pte.set_ppn2(paddr.ppn2());
            *entry = pte;
        },
    }
}

static mut ROOT_PAGE_TABLE: Option<*mut PageTable> = None;


// This is from 10.3.2. Virtual Address Translation Process
/// Returns
/// If find the entry and is valid, returns the corresponding PageTableEntry inside Ok()
/// If entry is invalid, returns a ptr to the entry, inside Err()
pub fn get_page(va: Sv39VirtualAddress) -> Result<&'static mut PageTableEntry, *mut PageTableEntry> {
    // Step 1.
    let satp = SATP(csrr!("satp"));
    let mut current_pte_addr = satp.ppn() * 4096;
    let mut level = 3;
    assert!((get_mode() == PrivilegeLevel::Supervisor || get_mode() == PrivilegeLevel::User));

    let leaf_pte = loop {
        dbg!(level, current_pte_addr);
        if level == 0 {
            panic!("Not found ?")
        }
        level -= 1;
        // Step 2.
        let pt = unsafe {&mut *(current_pte_addr as *mut PageTable)};
        let mut pte = &mut pt.entries[(va.vpn(level) * (core::mem::size_of::<Sv39VirtualAddress>() as u64)) as usize];
        // CAUTION! If accessing pte violates a PMA or PMP check, raise an access-fault exception corresponding to the
        // original access type
        // let pte = unsafe { *(pte_addr as *mut PageTableEntry) };
        // Step 3.
        if !pte.is_valid() || (!pte.is_readable() && pte.is_writable()) {
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
        current_pte_addr = match level {
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
    crate::println!("Initialising paging...");
    // unsafe{dbg!(csrr!("mstatus"))};
    unsafe { enter_mode(PrivilegeLevel::Supervisor) };
    // unsafe{dbg!(csrr!("mstatus"))};
    let mut satp = SATP(0);
    satp.set_mode(PagingModes::Sv39.satp());
    // satp.set_asid();
    let root_page_table_ptr = crate::heap::kalloc(1).unwrap() as *mut PageTable;
    let mut root_page_table = PageTable::new();
    for page_table in &mut root_page_table.entries {
        page_table.set_readable(1);
        page_table.set_writable(1);
        page_table.set_executable(1);
    }
    unsafe { *(root_page_table_ptr) = root_page_table };
    unsafe { ROOT_PAGE_TABLE.replace(root_page_table_ptr) };
    satp.set_ppn((root_page_table_ptr as u64) >> 12); // 2^12=4096
    dbg!(root_page_table_ptr);
    unsafe { csrw!("satp", satp.0) };
    unsafe { csrr!("satp") };
    let vaddr = Sv39VirtualAddress::new(crate::traps::save_context as _);
    let paddr = Sv39PhysicalAddress::new(kalloc(1).unwrap() as u64);
    let mut flags = PageTableEntryFlags(0b1111); // XWRV
    unsafe{map(vaddr, paddr, flags)}
    let pte = get_page(vaddr);
    crate::dbg!(pte);
    crate::dbg_bits!(pte.unwrap().0);
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
