use crate::*;

pub fn init() {
    loop {
        println!("Initialising PLIC...");
        wfi();
        for i in 0..1384171 {}
    }
    todo!()
}