use std::fmt::Debug;

use csr::{CsrID, CsrValue};
use instructions::Instruction;
use mem::MemoryMap;

use crate::*;

pub mod reg;
pub mod csr;
pub mod instructions;
pub mod raw_instructions;

pub struct CPU {
    pub regs: [uguest; 32],
    pub csrs: [CsrValue; 4096],
    pub privilege_level: PrivilegeLevel,
    pub pc: uguest,
}
impl CPU {
    pub fn reg(&mut self, reg: reg::Reg) -> &mut uguest {
        &mut self.regs[reg as usize]
    }
    pub fn csr(&mut self, csr: CsrID) -> &mut CsrValue {
        &mut self.csrs[csr.get() as usize]
    }
}
impl Debug for CPU {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut struc = f.debug_struct("CPU");
        for (i,reg) in self.regs.iter().enumerate() {
            struc.field(format!("{:?}", reg::Reg::new(i as u8)).as_str(), reg);
        }
        struc.field("pc", &self.pc).finish()
    }
}
impl Default for CPU {
    fn default() -> Self {
        Self { regs: Default::default(), pc: mem::MemMap::DRAM.base(), csrs: [CsrValue(0); 4096], privilege_level: PrivilegeLevel::Machine }
    }
}
pub enum PrivilegeLevel {
    User = 0,
    Supervisor = 1,
    Reserved = 2,
    Machine = 3,
}