#![no_std]
// For format msg in panic info
#![allow(internal_features)]
#![feature(fmt_internals)]

#![cfg_attr(debug_assertions, allow(unused, dead_code))]
// So that we make sure unsafe code is wrapped into `unsafe` even if the function is marked as unsafe.
// This is usefull, because we can see *clearly* where our unsafe blocks are.
#![warn(unsafe_op_in_unsafe_fn)]
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
pub mod riscv;
pub mod pmp;

pub use heap::kalloc;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
    print!("PANIC: ");
    if let Some(loc) = info.location() {
        print!("at {}:{}:{} ", loc.file(), loc.line(), loc.column());
    };
    if let Some(msg) = info.message().as_str() {
        println!("{}", msg);
    } else {
        println!("{}", alloc::string::ToString::to_string(&info.message()));
    }
    loop {}
}

