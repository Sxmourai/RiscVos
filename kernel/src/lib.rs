#![no_std]
#![cfg_attr(debug_assertions, allow(unused, dead_code))]


pub mod tests;
pub mod uart;
pub mod console;
pub mod heap;
pub mod paging;
pub mod traps;
pub mod plic;
pub mod riscv_utils;

extern crate alloc;
#[panic_handler]
fn _panic(_info: &core::panic::PanicInfo) -> ! {
    dbg!(_info);
    loop {}
}