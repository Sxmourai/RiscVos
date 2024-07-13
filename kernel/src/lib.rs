#![no_std]
#![cfg_attr(debug_assertions, allow(unused, dead_code))]

pub mod tests;
pub mod uart;
pub mod console;
pub mod heap;

extern crate alloc;