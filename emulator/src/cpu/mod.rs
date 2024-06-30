use instructions::Instruction;
use reg::{Register, RegisterValue};

use crate::r;

pub mod reg;
pub mod instructions;
pub mod raw_instructions;

#[repr(C)] // To be sure that it's like an array in memory
#[derive(Debug)]
pub struct CPU {
    pub zero:u32,
    pub ra:u32,
    pub sp:u32,
    pub gp:u32,
    pub tp:u32,
    pub t0:u32,
    pub t1:u32,
    pub t2:u32,
    pub s0:u32,
    pub s1:u32,
    pub a0:u32,
    pub a1:u32,
    pub a2:u32,
    pub a3:u32,
    pub a4:u32,
    pub a5:u32,
    pub a6:u32,
    pub a7:u32,
    pub s2:u32,
    pub s3:u32,
    pub s4:u32,
    pub s5:u32,
    pub s6:u32,
    pub s7:u32,
    pub s8:u32,
    pub s9:u32,
    pub s10:u32,
    pub s11:u32,
    pub t3:u32,
    pub t4:u32,
    pub t5:u32,
    pub t6:u32,


    pub pc: u32,
}
impl CPU {
    pub fn new() -> Self {
        Self {
            zero: 0, ra: 0, sp: 0, gp: 0, tp: 0, t0: 0, t1: 0, t2: 0, s0: 0, s1: 0, a0: 0, a1: 0, a2: 0, a3: 0, a4: 0, a5: 0, a6: 0, a7: 0, s2: 0, s3: 0, s4: 0, s5: 0, s6: 0, s7: 0, s8: 0, s9: 0, s10: 0, s11: 0, t3: 0, t4: 0, t5: 0, t6: 0,
            pc: 0, 
        }
    }
    pub fn as_array(&self) -> &[u32] {
        unsafe{std::slice::from_raw_parts(std::ptr::addr_of!(self.zero), 32)}
    }
    pub fn as_array_mut(&mut self) -> &mut [u32] {
        unsafe{std::slice::from_raw_parts_mut(std::ptr::addr_of_mut!(self.zero), 32)}
    }
}
impl std::ops::Index<usize> for CPU {
    type Output = RegisterValue;

    fn index(&self, index: usize) -> &Self::Output {
        &self.as_array()[index]
    }
}
impl std::ops::Index<Register> for CPU {
    type Output = RegisterValue;

    fn index(&self, index: Register) -> &Self::Output {
        &self.as_array()[index.0 as usize]
    }
}
impl std::ops::IndexMut<Register> for CPU {
    fn index_mut(&mut self, index: Register) -> &mut Self::Output {
        &mut self.as_array_mut()[index.0 as usize]
    }
}
