use bitfield::bitfield;

use crate::{csrw, dbg};

pub enum PMPConfigRegFormatA {
    Disabled = 0,
    /// Top of range
    /// If TOR is selected, the associated address register forms the top of the address range, and the
    /// preceding PMP address register forms the bottom of the address range. If PMP entry i's A field is set to
    /// TOR, the entry matches any address y such that pmpaddri-1≤y<pmpaddri (irrespective of the value of
    /// pmpcfgi-1). If PMP entry 0’s A field is set to TOR, zero is used for the lower bound, and so it matches
    /// any address y<pmpaddr0.
    TOR = 1,
    /// Naturally aligned four-byte region
    NA4 = 2, 
    /// Table 19. NAPOT range encoding in PMP address and configuration registers.
    /// pmpaddr       pmpcfg.A  Match type and size
    /// yyyy…yyyy    - NA4    -  4-byte NAPOT range
    /// yyyy…yyy0    - NAPOT  -  8-byte NAPOT range
    /// yyyy…yy01    - NAPOT  -  16-byte NAPOT range
    /// yyyy…y011    - NAPOT  -  32-byte NAPOT range
    /// …            - …      -  …
    /// yy01…1111    - NAPOT  -  2^XLEN-byte NAPOT range
    /// y011…1111    - NAPOT  -  2^XLEN+1-byte NAPOT range
    /// 0111…1111    - NAPOT  -  2^XLEN+2-byte NAPOT range
    /// 1111…1111    - NAPOT  -  2^XLEN+3-byte NAPOT range
    NAPOT = 3, // Naturally aligned power-of-two region, >= 8 bytes
}

bitfield! {
    pub struct PMPConfigRegFormat(u64);
    impl Debug;
    r, set_r: 0;
    w, set_w: 1;
    x, set_x: 2;
    a, set_a: 4, 3;
    /// The L bit indicates that the PMP entry is locked, i.e., writes to the configuration register and associated
    /// address registers are ignored. Locked PMP entries remain locked until the hart is reset. If PMP entry i
    /// is locked, writes to pmpicfg and pmpaddri are ignored. Additionally, if PMP entry i is locked and 
    /// pmpicfg.A is set to TOR, writes to pmpaddri-1 are ignored.
    l, set_l: 7;
}

#[macro_export]
macro_rules! write_pmpaddr {
    ($i: expr, $addr: ident) => {
        $crate::csrw!(concat!("pmpaddr",i), addr>>2); // 56-63 bits are zeros, but they will kindly be ignored =) (WARL)
    };
}
#[macro_export]
macro_rules! pmpcfg_r {
    ($i: expr) => {{
        assert!($i%2==0, "For RV64, the odd-numbered configuration registers, pmpcfg1, pmpcfg3, …, pmpcfg15, are illegal");
        $crate::csrr!(concat!("pmpcfg", $i))
    }};
}
#[macro_export]
macro_rules! pmpcfg_w {
    ($i: expr, $val: expr) => {{
        assert!($i%2==0, "For RV64, the odd-numbered configuration registers, pmpcfg1, pmpcfg3, …, pmpcfg15, are illegal");
        $crate::csrw!(concat!("pmpcfg", $i), $val)
    }};
}

fn get_pmp_i_cfg(pmpcfgi: usize, i: usize) -> usize {
    return pmpcfgi & (0xFF<<i)
}

pub fn init() {
    // Give access to full memory + needed by QEMU
    let mut cfg = PMPConfigRegFormat(0b111); // RWX
    cfg.set_a(PMPConfigRegFormatA::TOR as u64);
    unsafe{pmpcfg_w!(2, cfg.0)};
    dbg!(unsafe{pmpcfg_r!(2)});
}