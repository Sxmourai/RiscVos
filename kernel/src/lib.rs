#![no_std]

#![cfg_attr(debug_assertions, allow(unused, dead_code))]
#![warn(unused_must_use)]
#![warn(static_mut_refs)]
#![cfg_attr(not(debug_assertions), warn(clippy::unwrap_used))]
// So that we make sure unsafe code is wrapped into `unsafe` even if the function is marked as unsafe.
// This is usefull, because we can see *clearly* where our unsafe blocks are.
#![warn(unsafe_op_in_unsafe_fn)]
// I like using .clone sometimes to be extra verbose
#![allow(clippy::clone_on_copy)]
#[cfg(not(target_arch="riscv64"))]
compile_error!("Target arch should be riscv 64 !");
extern crate alloc;
pub use log::{trace,debug,info,warn,error};

pub mod riscv;
pub use riscv::*;
pub mod tests;

pub mod uart;
pub mod logging;

pub mod heap;
pub use heap::{kalloc,kmalloc};
pub mod paging;
pub mod pmp;

pub mod traps;
pub mod plic;
pub mod clint;
pub mod virtio;

pub mod thread;

pub fn main_loop() {    
    let mut blk = match unsafe { crate::virtio::VIRTIO_DEVICES[7].as_mut().unwrap() } {
        virtio::VirtIODevicePtr::Block(blk) => blk,
        _ => todo!(),
    };
    let mut buffer = alloc::vec![0u8; 1024];
    blk.read(0, &mut buffer);
    dbg!(buffer);
	loop {
        riscv::wfi()
    }
}