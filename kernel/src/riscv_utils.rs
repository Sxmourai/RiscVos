#[macro_export]
macro_rules! csrr {
    ($reg: expr) => {{
        csrr!($reg, u64)
    }};
    ($reg: expr, $res: ty) => {{
        let mut res: $res;
        unsafe{core::arch::asm!(concat!("csrr {}, ", $reg), out(reg) res)};
        res
    }};
}
#[macro_export]
macro_rules! csrw {
    ($reg: expr, $val: expr) => {{
        csrw!($reg, $val, u64)
    }};
    ($reg: expr, $val: expr, $res: ty) => {{
        core::arch::asm!(concat!("csrw ", $reg, ", {}"), in(reg) $val);
    }};
}