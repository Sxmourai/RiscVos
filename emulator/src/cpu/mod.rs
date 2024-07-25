use std::fmt::Debug;

use instructions::Instruction;
use mem::MemoryMap;

use crate::*;

pub mod reg;
pub mod instructions;
pub mod raw_instructions;

pub struct CPU {
    pub regs: [uguest; 32],
    pub pc: uguest,
}
impl CPU {
    pub fn reg(&mut self, reg: reg::Reg) -> &mut uguest {
        &mut self.regs[reg as usize]
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
        Self { regs: Default::default(), pc: mem::MemMap::DRAM.base() }
    }
}