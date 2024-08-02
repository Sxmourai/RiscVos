// We can now access cpu registers using zero, s0, a0 etc... See cpu/reg.rs 
use crate::cpu::reg::Reg;
use color_eyre::eyre::{Context, ContextCompat};
use color_eyre::Result;
use cpu::instructions::Instruction16;

use crate::*;
use crate::cpu::instructions::Instruction;

pub struct VM {
    pub mem: mem::Memory,
    pub cpu: crate::cpu::CPU,
}
impl VM {
    pub fn new(program: Vec<u8>) -> Self {
        crate::cpu::raw_instructions::set_instructions_funcs();
        Self {
            mem: mem::Memory::with_program(program),
            cpu: crate::cpu::CPU::default(),
        }
    }
    
    pub fn run(&mut self) -> color_eyre::Result<()> {
        loop {
            print!("{:x}", self.cpu.pc);
            // Fetch
            let raw_instruction = self.mem.get::<u32>(self.cpu.pc).context("Out of bounds")?;
            if raw_instruction == 0 {
                println!("Didn't enter in loop !");
                return Ok(()) // Don't pollute stdout, for now
            }
            let instructions = Instruction::new(raw_instruction).context("The program didn't enter in a end-loop ! This would've led to UB")?;
            if let Some(fst) = instructions.1 {
                let snd = match instructions.0 {
                    Instruction::Base(_) => unreachable!(),
                    Instruction::Compressed(c) => c,
                };
                todo!();
                // Execute
                // println!(" - {}", fst);
                // let (_name, _fmt, _mask, fun) = crate::cpu::instructions::find_instruction16_desc(fst);
                // fun(self, fst);
                // self.cpu.pc += core::mem::size_of::<Instruction16>() as uguest;
                // *self.cpu.reg(Reg::zero) = 0; // Currently we need to set it manually
                // println!(" - {}", snd);
                // let (_name, _fmt, _mask, fun) = crate::cpu::instructions::find_instruction16_desc(snd);
                // fun(self, snd);
                self.cpu.pc += core::mem::size_of::<Instruction16>() as uguest;
            } else {
                let instruction = match instructions.0 {
                    Instruction::Base(b) => b,
                    Instruction::Compressed(_) => unreachable!(),
                };
                // Execute
                println!(" - {}", instruction);
                let (_name, _fmt, _mask, fun) = crate::cpu::raw_instructions::find_instruction32_desc(instruction);
                fun(self, instruction);
                self.cpu.pc += core::mem::size_of::<u32>() as uguest;
            }
            *self.cpu.reg(Reg::zero) = 0; // Currently we need to set it manually
            #[cfg(debug_assertions)]
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        dbg!(&self.cpu);
        Ok(())
    }
    // pub fn disasm(&mut self, program: Vec<u8>) -> color_eyre::Result<()> {
    //     for i in (0..program.len()).step_by(core::mem::size_of::<Instruction>()) {
    //         // Fetch
    //         let instruction = Instruction::new(self.mem.get::<_>(self.cpu.pc).unwrap()).context("The program didn't enter in a end-loop ! This would've led to UB")?;
    //         // Execute
    //         println!("{}\t - ", instruction);
    //     }
    //     Ok(())
    // }
}
impl std::fmt::Debug for VM {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VM").field("cpu", &self.cpu).finish()
    }
}
impl std::fmt::Display for VM {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write;
        let mut regs = String::new();
        for (i,reg_value) in self.cpu.regs.iter().enumerate() {
            let reg_name = cpu::reg::REGS[i];
            write!(regs, "{}: {}\t\t", reg_name, reg_value)?;
            if (i+1)%4 == 0 {
                writeln!(regs);
            }
        }
        let csrs = format_args!("todo");
        f.write_fmt(format_args!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ REGISTERS ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
{}
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ CSRs ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
{}
        ", regs, csrs))
    }
}

#[cfg(debug_assertions)]
pub static mut MAIN_VM: Option<VM> = None;

#[cfg(debug_assertions)]
/// Sets up a pretty print of the vm state when panicking
fn setup_dbg_vm(vm: VM) -> &'static mut VM {
    unsafe {
        let _ = MAIN_VM.replace(vm);
        let color_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            println!("VM: {}", MAIN_VM.as_mut().unwrap());
            color_hook(info)
        }));
        MAIN_VM.as_mut().unwrap()
    }
}

pub fn run(program: Vec<u8>) -> Result<()> {
    let mut vm = VM::new(program);
    let vm = setup_dbg_vm(vm);
    match vm.run() {
        Ok(_) => Ok(()),
        Err(err) => {
            dbg!(vm);
            Err(err)
        },
    }
}

// let (s1, has_s2) = match instruction.s1() {
//     (crate::cpu::instructions::Destination::CpuRegister(reg), has_s2) => {
//         (*self.cpu.reg(reg), has_s2)
//     }
//     (crate::cpu::instructions::Destination::Immediate(imm), has_s2) => (imm as uguest, has_s2),
// };
// let s2 = if has_s2 {
//     match instruction.s2() {
//         crate::cpu::instructions::Destination::CpuRegister(reg) => *self.cpu.reg(reg),
//         crate::cpu::instructions::Destination::Immediate(imm) => imm as uguest,
//     }
// } else {
//     0
// };