use core::cell::OnceCell;

use bitfield::bitfield;

use crate::{csrr, csrw, dbg, heap::kalloc};

// PMP -> Physical memory protection
// PMP checks are only in Supervised or User mode
//////////// Or data accesses in Machine mode (our mode for now) when the MPRV bit is set
pub static mut MEMORY_PROTECTION: OnceCell<PMP> = OnceCell::new();

// Memory Management Unit
pub struct PMP {
    page_tables: [Table; 512],
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

    #[allow(non_snake_case)]
    pub SD,  set_SD: 63, 62;

    // Controls endianness (User, Machine, Supervisor)
    #[allow(non_snake_case)]
    pub UBE,  set_UBE: 7, 6;
    #[allow(non_snake_case)]
    pub MBE,  set_MBE: 38, 37;
    #[allow(non_snake_case)]
    pub SBE,  set_SBE: 37, 36;
    // SXL and UXL control value of XLEN for S and U mode
    // The encoding of these fields is the same as the MXL field of misa, shown in
    // Table 9. The effective XLEN in S-mode and U-mode are termed SXLEN and UXLEN, respectively.
    // Not our case but: When MXLEN=32, the SXL and UXL fields do not exist, and SXLEN=32 and UXLEN=32.
    // the set of legal values that the UXL field may assume excludes those that
    // would cause UXLEN > SXLEN (== is fine)
    pub SXL, set_SXL: 36, 34; // 2 bits !
    pub UXL, set_UXL: 34, 32; // 2 bits !

    // Trap SRET
    pub TSR,  set_TSR: 23, 22;
    // (Timeout Wait)
    pub TW,   set_TW: 22, 21;
    // Trap Virtual Memory, used for virtualisation
    pub TVM,  set_TVM: 21, 20;
    // The MXR (Make eXecutable Readable) bit modifies the privilege with which loads access virtual
    // memory. When MXR=0, only loads from pages marked readable (R=1 in Figure 59) will succeed.
    // When MXR=1, loads from pages marked either readable or executable (R=1 or X=1) will succeed. MXR
    // has no effect when page-based virtual memory is not in effect. MXR is read-only 0 if S-mode is not
    // supported
    pub MXR,  set_MXR: 20, 19;
    // permit Supervisor User Memory access bit modifies the privilege with which S-mode loads
    // and stores access virtual memory. When SUM=0, S-mode memory accesses to pages that are
    // accessible by U-mode (U=1 in Figure 59) will fault. When SUM=1, these accesses are permitted. SUM
    // has no effect when page-based virtual memory is not in effect. Note that, while SUM is ordinarily
    // ignored when not executing in S-mode, it is in effect when MPRV=1 and MPP=S. SUM is read-only 0 if
    // S-mode is not supported or if satp.MODE is read-only 0.
    pub SUM,  set_SUM: 19, 18;
    // 3.1.6.3. Memory Privilege in mstatus Register: Modify privilege
    //  When MPRV=0, loads and stores behave as normal, using the translation and
    // protection mechanisms of the current privilege mode. When MPRV=1, load and store memory
    // addresses are translated and protected, and endianness is applied, as though the current privilege
    // mode were set to MPP. Instruction address-translation and protection are unaffected by the setting of
    // MPRV. MPRV is read-only 0 if U-mode is not supported.
    // An MRET or SRET instruction that changes the privilege mode to a mode less privileged than M also
    // sets MPRV=0.
    pub MPRV, set_MPRV: 18, 17;
    //todo Interesting for speeding up context switches
    pub XS,   set_XS: 17, 5; // 2 bits !
    pub FS,   set_FS: 15, 13; // 2 bits !
    //  When a trap is taken from privilege mode y
    // into privilege mode x, xPIE is set to the value of xIE; xIE is set to 0; and xPP is set to y
    pub MPP,  set_MPP: 13, 11; // 2 bits !, holds previous privilege mode for machine traps
    // M-mode software can determine whether a privilege mode is implemented by writing that
    // mode to MPP then reading it back.

    pub VS,   set_VS: 11, 9; // 2 bits !
    pub SPP,  set_SPP: 9, 8; // holds previous privilege mode for supervisor traps (cuz it can only be User, only 1 bit)
    pub MPIE, set_MPIE: 8, 7;
    pub SPIE, set_SPIE: 6, 5;
    pub MIE,  set_MIE: 4, 3; // Global interrupt-enable bits supervisor mode
    pub SIE,  set_SIE: 2, 1; // Global interrupt-enable bits machine mode
}

// pub static mut PageTables
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PrivilegeLevel {
    User = 0,       // Abbr: U
    Supervisor = 1, // Abbr: S
    Reserved = 2,   // Abbr:
    Machine = 3,    // Abbr: M
}
pub unsafe fn enter_mode(priv_level: PrivilegeLevel) {
    let mstatus = csrr!("mstatus");
    csrw!(
        "mstatus",
        mstatus & !(0b11 << 11) | (priv_level as u64) << 11
    )
}
pub fn get_mode() -> PrivilegeLevel {
    match MSTATUS(csrr!("mstatus")).MPP() {
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
        Sv39PhysicalAddress((self.0<<2)&!(0x3FF))// Get only bits 56-10 and shift by 2 to right 
    }
    // If Read or Write or Execute bit is set, then it is a leaf, else it's a branch
    pub fn is_leaf(self) -> bool {
        self.0 & 0b1110 != 0
    }
}
// Can't derive cuz using bitfield
impl Clone for PageTableEntry {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl Copy for PageTableEntry {}

bitfield! {
    // Only 56 bits
    pub struct Sv39PhysicalAddress(u64);
    impl Debug;

    pub page_offset, set_page_offset: 12, 0;

    pub ppn0, set_ppn0: 21, 12;
    pub ppn1, set_ppn1: 30, 21;
    pub ppn2, set_ppn2: 56, 30;
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
    pub fn vpn(self, vpni: u64) -> u64 {
        assert!(vpni <= 3);
        (self.0 >> (9 * vpni)) & 0x1FF
    }
}
impl Clone for Sv39VirtualAddress {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl Copy for Sv39VirtualAddress {}

pub struct PageTable {
    pub entries: [PageTableEntry; 512],
}
impl PageTable {
    pub fn new() -> Self {
        Self {
            entries: [PageTableEntry(0b1); 512],
        }
    }
}
// Will return result
// Automatically gets root page table, and page-level
pub unsafe fn map(vaddr: Sv39VirtualAddress, paddr: Sv39PhysicalAddress, flags: PageTableEntry) {
    assert!(flags.is_leaf());
    // let vpn = [
    //     // VPN[2] = vaddr[38:30]
    //     (vaddr >> 30) & 0x1ff,
    //     // VPN[1] = vaddr[29:21]
    //     (vaddr >> 21) & 0x1ff,
    //     // VPN[0] = vaddr[20:12]
    //     (vaddr >> 12) & 0x1ff,
    // ];
    // let ppn = [
    //     // PPN[2] = paddr[55:30]
    //     (paddr >> 30) & 0x3ff_ffff,
    //     // PPN[1] = paddr[29:21]
    //     (paddr >> 21) & 0x1ff,
    //     // PPN[0] = paddr[20:12]
    //     (paddr >> 12) & 0x1ff,
    // ];
    let level = 3; // Manually set level for now
    let mut current_page_table = unsafe { &mut *(ROOT_PAGE_TABLE.unwrap()) };
    for i in (0..level) {
        todo!();
        // let page = vpn[i];
        // let frame = ppn[i]; // Probably not right name, but comes from x86 and makes sens
        // let entry = &mut current_page_table.entries[page];
        // // If no entry is in table, we have to create one
        // if entry.valid() == 0 {
        //     let entry_page = kalloc(1).unwrap();

        // }
    }
}

static mut ROOT_PAGE_TABLE: Option<*mut PageTable> = None;

// This is from 10.3.2. Virtual Address Translation Process
pub fn virtual_to_physical(va: Sv39VirtualAddress) -> Sv39PhysicalAddress {
    // Step 1.
    let satp = SATP(csrr!("satp"));
    let mut current_pte_addr = satp.ppn() * 4096;
    let mut level = 3;
    assert!((get_mode() == PrivilegeLevel::Supervisor || get_mode() == PrivilegeLevel::User));

    let leaf_pte = loop {
        if level == 0 {
            panic!("Not found ?")
        }
        level -= 1;
        // Step 2.
        let pte_addr = current_pte_addr + va.vpn(level) * (core::mem::size_of::<Sv39VirtualAddress>() as u64);
        // CAUTION! If accessing pte violates a PMA or PMP check, raise an access-fault exception corresponding to the
        // original access type
        let pte = unsafe { *(pte_addr as *mut PageTableEntry) };
        // Step 3.
        if !pte.is_valid() || (!pte.is_readable() && pte.is_writable()) {
            // || pte.any_reserved_combination
            panic!("Page fault: trying to load from invalid entry !");
        }
        // Step 4.
        if pte.is_leaf() {
            // Step 5.
            break pte;
        }
        current_pte_addr = pte.parse_ppn().0*4096;
    };


    let mut pa = Sv39PhysicalAddress(0);
    pa
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
    dbg!(satp, root_page_table_ptr);
    unsafe { csrw!("satp", satp.0) };
    unsafe { dbg!(csrr!("satp")) };
    unsafe { core::ptr::write_volatile(0x100 as *mut u8, 10) }

    // unsafe{core::arch::asm!("csrw satp, {}", in(reg) satp)};
}

#[repr(usize)]
#[derive(Copy, Clone)]
pub enum EntryBits {
    None = 0,
    Valid = 1 << 0,
    Read = 1 << 1,
    Write = 1 << 2,
    Execute = 1 << 3,
    User = 1 << 4,
    Global = 1 << 5,
    Access = 1 << 6,
    Dirty = 1 << 7,

    // Convenience combinations
    ReadWrite = 1 << 1 | 1 << 2,
    ReadExecute = 1 << 1 | 1 << 3,
    ReadWriteExecute = 1 << 1 | 1 << 2 | 1 << 3,

    // User Convenience Combinations
    UserReadWrite = 1 << 1 | 1 << 2 | 1 << 4,
    UserReadExecute = 1 << 1 | 1 << 3 | 1 << 4,
    UserReadWriteExecute = 1 << 1 | 1 << 2 | 1 << 3 | 1 << 4,
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
