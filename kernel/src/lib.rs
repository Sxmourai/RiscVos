#![no_std]
#![cfg_attr(debug_assertions, allow(unused, dead_code))]
#[cfg(not(target_arch="riscv64"))]
compile_error!("Target arch should be riscv 64 !");
extern crate alloc;


pub mod tests;
pub mod uart;
pub mod console;
pub mod heap;
pub mod paging;
pub mod traps;
pub mod plic;
pub mod riscv_utils;

pub use heap::kalloc;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo<'_>) -> ! {
    dbg!(_info);
    loop {}
}

// Should be unsafe, because it could stop os if no interrupts ?
pub fn wfi() {
    unsafe {
        core::arch::asm!("wfi"); // "hlt" in x86
    }
}
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
impl SATP {
    pub fn read() -> Self {
        Self(csrr!("satp"))
    }
}
bitfield::bitfield! {
    pub struct MSTATUS(u64);
    impl Debug;

    pub sd,  set_sd: 62;

    // Controls endianness (User, Machine, Supervisor)
    pub ube,  set_ube: 6;
    pub mbe,  set_mbe: 37;
    pub sbe,  set_sbe: 36;
    // SXL and UXL control value of XLEN for S and U mode
    // The encoding of these fields is the same as the MXL field of misa, shown in
    // Table 9. The effective XLEN in S-mode and U-mode are termed SXLEN and UXLEN, respectively.
    // Not our case but: When MXLEN=32, the SXL and UXL fields do not exist, and SXLEN=32 and UXLEN=32.
    // the set of legal values that the UXL field may assume excludes those that
    // would cause UXLEN > SXLEN (== is fine)
    pub sxl, set_sxl: 36, 34; // 2 bits !
    pub uxl, set_uxl: 34, 32; // 2 bits !

    // Trap SRET
    pub tsr,  set_tsr: 22;
    // (Timeout Wait)
    pub tw,   set_tw: 21;
    // Trap Virtual Memory, used for virtualisation
    pub tvm,  set_tvm: 20;
    // The MXR (Make eXecutable Readable) bit modifies the privilege with which loads access virtual
    // memory. When MXR=0, only loads from pages marked readable (R=1 in Figure 59) will succeed.
    // When MXR=1, loads from pages marked either readable or executable (R=1 or X=1) will succeed. MXR
    // has no effect when page-based virtual memory is not in effect. MXR is read-only 0 if S-mode is not
    // supported
    pub mxr,  set_mxr: 19;
    // permit Supervisor User Memory access bit modifies the privilege with which S-mode loads
    // and stores access virtual memory. When SUM=0, S-mode memory accesses to pages that are
    // accessible by U-mode (U=1 in Figure 59) will fault. When SUM=1, these accesses are permitted. SUM
    // has no effect when page-based virtual memory is not in effect. Note that, while SUM is ordinarily
    // ignored when not executing in S-mode, it is in effect when MPRV=1 and MPP=S. SUM is read-only 0 if
    // S-mode is not supported or if satp.MODE is read-only 0.
    pub sum,  set_sum: 18;
    // 3.1.6.3. Memory Privilege in mstatus Register: Modify privilege
    //  When MPRV=0, loads and stores behave as normal, using the translation and
    // protection mechanisms of the current privilege mode. When MPRV=1, load and store memory
    // addresses are translated and protected, and endianness is applied, as though the current privilege
    // mode were set to MPP. Instruction address-translation and protection are unaffected by the setting of
    // MPRV. MPRV is read-only 0 if U-mode is not supported.
    // An MRET or SRET instruction that changes the privilege mode to a mode less privileged than M also
    // sets MPRV=0.
    pub mprv, set_mprv: 17;
    //todo Interesting for speeding up context switches
    pub xs,   set_xs: 17, 5; // 2 bits !
    pub fs,   set_fs: 15, 13; // 2 bits !
    //  When a trap is taken from privilege mode y
    // into privilege mode x, xPIE is set to the value of xIE; xIE is set to 0; and xPP is set to y
    pub mpp,  set_mpp: 13, 11; // 2 bits !, holds previous privilege mode for machine traps
    // M-mode software can determine whether a privilege mode is implemented by writing that
    // mode to MPP then reading it back.

    pub vs,   set_vs: 11, 9; // 2 bits !
    pub spp,  set_spp: 8; // holds previous privilege mode for supervisor traps (cuz it can only be User, only 1 bit)
    pub mpie, set_mpie: 7;
    pub spie, set_spie: 5;
    pub mie,  set_mie: 3; // Global interrupt-enable bits supervisor mode
    pub sie,  set_sie: 1; // Global interrupt-enable bits machine mode
}
impl MSTATUS {
    pub fn read() -> Self {
        Self(csrr!("mstatus"))
    }
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

pub unsafe fn assert_mstatus() {
    let mstatus = csrr!("mstatus");
    unsafe { enter_mode(PrivilegeLevel::Machine) }
    if mstatus != csrr!("mstatus") {
        dbg!(mstatus, MSTATUS::read());
    }
} 


pub fn spin_loop() -> ! {
    loop {wfi()}
}