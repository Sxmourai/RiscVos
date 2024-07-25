// We can now access cpu registers using zero, s0, a0 etc... See cpu/reg.rs 
use crate::cpu::reg::Reg;
use color_eyre::eyre::{Context, ContextCompat};
use cpu::instructions::INSTRUCTIONS_MASKS;

use crate::*;
use crate::cpu::instructions::Instruction;

pub struct VM {
    pub mem: mem::Memory,
    pub cpu: crate::cpu::CPU,
}
impl VM {
    pub fn new(program: Vec<u8>) -> Self {
        crate::cpu::instructions::set_instructions_funcs();
        Self {
            mem: mem::Memory::with_program(program),
            cpu: crate::cpu::CPU::default(),
        }
    }
    
    pub fn run(&mut self) -> color_eyre::Result<()> {
        loop {
            // Fetch
            let raw_instruction = self.mem.get::<u32>(self.cpu.pc).context("Out of bounds")?;
            if raw_instruction == 0 {
                println!("Didn't enter in loop !");
                return Ok(()) // Don't pollute stdout, for now
            }
            let instruction = Instruction::new(raw_instruction).context("The program didn't enter in a end-loop ! This would've led to UB")?;
            if instruction.opcode() == INSTRUCTIONS_MASKS[]
            // Execute
            println!("{}\t - ", instruction);
            let (_name, _fmt, _mask, fun) = crate::cpu::instructions::find_instruction_desc(instruction);

            let (s1, has_s2) = match instruction.s1() {
                (crate::cpu::instructions::Destination::CpuRegister(reg), has_s2) => {
                    (*self.cpu.reg(reg), has_s2)
                }
                (crate::cpu::instructions::Destination::Immediate(imm), has_s2) => (imm as uguest, has_s2),
            };
            let s2 = if has_s2 {
                match instruction.s2() {
                    crate::cpu::instructions::Destination::CpuRegister(reg) => *self.cpu.reg(reg),
                    crate::cpu::instructions::Destination::Immediate(imm) => imm as uguest,
                }
            } else {
                0
            };
            let d = fun(self, s1, s2);

            // println!("{}={}\t{}={}\t-> {}={}", inst.s1().0,s1,inst.s2(),s2,inst.destination(),d);
            match instruction.destination() {
                crate::cpu::instructions::Destination::CpuRegister(reg) => *self.cpu.reg(reg) = d,
                crate::cpu::instructions::Destination::Immediate(imm) => {
                    self.mem.set::<_>(imm as _, d).unwrap()
                }
            };
            self.cpu.pc += core::mem::size_of::<Instruction>() as uguest;
            *self.cpu.reg(Reg::zero) = 0; // Currently we need to set it manually
        }
        dbg!(&self.cpu);
        Ok(())
    }
    pub fn disasm(&mut self, program: Vec<u8>) -> color_eyre::Result<()> {
        for i in (0..program.len()).step_by(core::mem::size_of::<Instruction>()) {
            // Fetch
            let instruction = Instruction::new(self.mem.get::<_>(self.cpu.pc).unwrap()).context("The program didn't enter in a end-loop ! This would've led to UB")?;
            // Execute
            println!("{}\t - ", instruction);
        }
        Ok(())
    }
}

pub fn run(program: Vec<u8>) {
    VM::new(program).run().unwrap();
}
