use crate::*;

pub fn init() {
    loop {
        println!("Initialising PLIC...");
        riscv::wfi();
        for i in 0..1384171 {}
    }
    todo!()
}