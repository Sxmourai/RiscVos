pub enum SavedBy {
    Caller,
    Callee,
    None
}
pub const REGS: [&'static str; 32] = [
    "zero", "ra", "sp", "gp", "tp", "t0", "t1", "t2", 
    "s0", "s1", // or "fp"
    "a0", "a1", "a2", "a3", "a4", "a5", "a6", "a7", 
    "s2", "s3", "s4", "s5", "s6", "s7", "s8", "s9", "s10", "s11",
    "t3", "t4", "t5", "t6", 
    // "ft0", "ft1", "ft2", "ft3", "ft4", "ft5", "ft6", "ft7", 
    // "fs0", "fs1",
    // "fa0", "fa1",
    // "fa2","fa3","fa4","fa5","fa6","fa7",
    // "fs2", "fs3", "fs4", "fs5", "fs6", "fs7", "fs8", "fs9", "fs10", "fs11",
    // "ft8","ft9","ft10","ft11",
];
// Tested on https://godbolt.org and it compiles only to the number and not the match statement
#[macro_export]
macro_rules! r {
    // ($reg: a1) => {11};
    ($reg: expr) => {
        match $reg {
          "zero" => 0,
            "ra" => 1, 
            "sp" => 2, 
            "gp" => 3, 
            "tp" => 4, 
            "t0" => 5, 
            "t1" => 6, 
            "t2" => 7, 
            "s0" => 8, 
            "s1" => 9, 
            "a0" => 10, 
            "a1" => 11, 
            "a2" => 12, 
            "a3" => 13, 
            "a4" => 14, 
            "a5" => 15, 
            "a6" => 16, 
            "a7" => 17, 
            "s2" => 18, 
            "s3" => 19, 
            "s4" => 20,
            "s5" => 21, 
            "s6" => 22, 
            "s7" => 23, 
            "s8" => 24, 
            "s9" => 25, 
           "s10" => 26, 
           "s11" => 27, 
            "t3" => 28, 
            "t4" => 29, 
            "t5" => 30, 
            "t6" => 31, 
            _ => todo!(),
        }
    };
}

#[derive(Clone, Copy, Debug)]
pub struct Register(pub u32);
impl Register {
    pub fn new(reg: u32) -> Self {
        if reg as usize > REGS.len() {todo!()}
        let s = Self(reg);
        s
    }
}
impl std::fmt::Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        f.write_str(REGS[self.0 as usize])
    }
}

// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// #[repr(transparent)]
pub type RegisterValue = u32;