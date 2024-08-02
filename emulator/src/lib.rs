#![feature(const_mut_refs)] // For raw instructions
#![feature(vec_push_within_capacity)]
#![feature(pointer_byte_offsets)]
#![allow(dead_code, unused)]

pub mod args;
pub mod cpu;
pub mod mem;
pub mod vm;

#[allow(non_camel_case_types)]
pub type uguest = u64;
#[allow(non_camel_case_types)]
pub type iguest = i64;
pub type InstructionSize = u32;