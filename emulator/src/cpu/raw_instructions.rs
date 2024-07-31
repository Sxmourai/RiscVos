// Big thanks to https://www.eg.bucknell.edu/~csci206/riscv-converter/Annotated_RISCV_Card.pdf
// For more info about instructions https://projectf.io/posts/riscv-cheat-sheet/

use std::cell::OnceCell;

use color_eyre::Report;
use instruction_proc::instruction_r as r;
use instruction_proc::instruction_i as i;
use instruction_proc::instruction_s as s;
use instruction_proc::instruction_u as u;
use instruction_proc::instruction_j as j;
use instruction_proc::instruction_b as b;

pub type Rd = super::reg::Reg;
pub type Rs1 = super::reg::Reg;
pub type Rs2 = super::reg::Reg;
pub type Vs1 = uguest;
pub type Vs2 = uguest;
pub type Imm = u16;
pub type UImm = u32;

use crate::cpu::reg::Reg;
use crate::uguest;
use crate::cpu::CsrID;

use super::instructions::Instruction32;

use color_eyre::Result;

const fn _mask(opcode: u32, fun3: u32, fun7: u32) -> InstructionMask {
    InstructionMask(opcode | fun3 << 12 | fun7 << 25)
}
const fn desc(macro_out: (&'static str, Instruction32Format, InstructionFunction32), mask: InstructionMask) -> InstructionDescription32 {
    (macro_out.0, macro_out.1, mask, macro_out.2)
}

macro_rules! load {
    ($size: ty,$name: ident,$func3: expr) => {
        desc(i!($name, {
            vm.mem.get::<$size>((vs1+imm as uguest)).unwrap() as _
        }), _mask(0b0000011, $func3, 0b0))
        // pub fn $name(vm: &mut crate::vm::VM, instruction: Instruction) {
        //     let (vs1, imm, dest) = Instruction::parse_i(instruction);
        // }
    };
}
macro_rules! store {
    ($size: ty,$name: ident,$func3: expr) => {
        desc(s!($name, {
            vm.mem.set::<$size>((vs1+imm as uguest), (vs2&($size::MAX as uguest)) as _).unwrap();
        }), _mask(0b0100011, $func3, 0b0))
    };
}


macro_rules! op_i {
    ($name: ident, $operator: expr) => {
        i!($name, {
            let res = $operator(vs1, imm as uguest);
            res
        })
    };
}
macro_rules! op_r {
    ($name: ident, $func3: expr, $func7:expr, $operator: expr) => {
        desc(r!($name, {
            let res = $operator(vs1, vs2);
            res
        }), _mask(0b0110011, $func3, $func7))
    };
}

macro_rules! branch {
    ($name: ident, $func3: expr, $op: tt) => {
        desc(s!($name, {
            dbg!(vs1,vs2, imm);
            if $op(vs1,vs2) {
                vm.cpu.pc += vs2
            };
        }), _mask(0b1100011, $func3, 0b0))
    };
}

#[track_caller]
fn panic(a: uguest, b: uguest) -> uguest {
    todo!("{} {}", a,b)
}


#[derive(Clone, Copy, Debug)]
pub struct Instruction32Mask(pub u32);
#[derive(Clone, Copy, Debug)]
pub struct Instruction16Mask(pub u16);
#[derive(Debug, Clone, Copy)]
pub enum Instruction32Format {
    R,I,S,B,U,J,
}
#[derive(Debug, Clone, Copy)]
pub enum Instruction16Format {
    CR,  // Register
    CI,  // Immediate
    CSS, // Stack-relative Store
    CIW, // Wide Immediate
    CL,  // Load
    CS,  // Store
    CA,  // Arithmetic
    CB,  // Branch/Arithmetic
    CJ,  // Jump
}
pub type InstructionFunction32 = fn(&mut crate::vm::VM, super::instructions::Instruction32);
pub type InstructionDescription32 = (&'static str, Instruction32Format, Instruction32Mask, InstructionFunction32);
pub type InstructionFunction16 = fn(&mut crate::vm::VM, super::instructions::Instruction16);
pub type InstructionDescription16 = (&'static str, Instruction16Format, Instruction16Mask, InstructionFunction16);

/// Based on
/// Chapter 34. RV32/64G Instruction Set Listings
/// And https://www.eg.bucknell.edu/~csci206/riscv-converter/Annotated_RISCV_Card.pdf at beginning
pub const INSTRUCTIONS32: [InstructionDescription32; 60] = [
    load!(u8,  lb, 0),
    load!(u16, lh, 1),
    load!(u32, lw, 2),
    load!(u64, ld, 3),
    load!(u8,  lbu, 0+4),
    load!(u16, lhu, 1+4),
    load!(u32, lwu, 2+4),
    load!(u64, ldu, 3+4),
    // ("lbu",     InstructionFormat::I, _mask(0b0000011, 0b100, 0b0), lbu),
    // ("lhu",     InstructionFormat::I, _mask(0b0000011, 0b110, 0b0), lhu),
    // ("lwu",     InstructionFormat::I, _mask(0b0000011, 0b111, 0b0), lwu),
    
    desc(i!(fence, {
        todo!()
    }), _mask(0b0001111, 0b000, 0b0)),
    desc(i!(fencei, {
        todo!()
    }), _mask(0b0001111, 0b001, 0b0)),
    
    desc(op_i!(addi,core::ops::Add::add),    _mask(0b0010011, 0b000, 0b0)),
    desc(op_i!(slli,core::ops::Shl::shl),    _mask(0b0010011, 0b001, 0b0)),
    desc(op_i!(slti,panic),    _mask(0b0010011, 0b010, 0b0)),
    desc(op_i!(sltiu,panic), _mask(0b0010011, 0b011, 0b0)),
    desc(op_i!(xori,core::ops::BitXor::bitxor),    _mask(0b0010011, 0b100, 0b0)),
    desc(op_i!(srli,panic),    _mask(0b0010011, 0b101, 0b0000000)),
    desc(op_i!(srai,panic),    _mask(0b0010011, 0b101, 0b0100000)),
    desc(op_i!(ori,core::ops::BitOr::bitor),       _mask(0b0010011, 0b110, 0b0)),
    desc(op_i!(andi,core::ops::BitAnd::bitand),    _mask(0b0010011, 0b111, 0b0)),
    
    desc(u!(auipc, {vm.cpu.pc+(imm as uguest)}), _mask(0b0010111, 0b000, 0b0)),
    
    desc(i!(addiw, {vs1+imm as uguest}), _mask(0b0011011, 0b000, 0b0)),
    desc(i!(slliw, {(vs1)<<(imm as uguest)}), _mask(0b0011011, 0b001, 0b0000000)),
    desc(i!(srliw, {vs1>>imm as uguest}), _mask(0b0011011, 0b101, 0b0000000)),
    desc(i!(sraiw, {todo!("vs1>>>imm as uguest")}), _mask(0b0011011, 0b101, 0b0100000)),
    
    store!(u8,  sb, 0b000),
    store!(u16, sh, 0b001),
    store!(u32, sw, 0b010),
    store!(u64, sd, 0b011),
    
    op_r!(add,  0b000, 0b0000000, core::ops::Add::add),
    op_r!(sub,  0b000, 0b0100000, core::ops::Sub::sub),
    op_r!(sll,  0b001, 0b0000000, core::ops::Shl::shl),
    op_r!(slt,  0b010, 0b0000000, panic), // core::ops::Slt::slt
    op_r!(sltu, 0b011, 0b0000000, panic), // core::ops::Sltu::sltu
    op_r!(xor,  0b100, 0b0000000, core::ops::BitXor::bitxor),
    op_r!(srl,  0b101, 0b0000000, panic), // core::ops::srl::srl
    op_r!(sra,  0b101, 0b0100000, panic), // core::ops::sra::sra
    op_r!(or,   0b110, 0b0000000, core::ops::BitOr::bitor),
    op_r!(and,  0b111, 0b0000000, core::ops::BitAnd::bitand),
    
    desc(u!(lui, {imm as uguest}),   _mask(0b0110111, 0b0, 0b0)),
    
    desc(r!(addw, {todo!()}),  _mask(0b0111011, 0b000, 0b0000000)),
    desc(r!(subw, {todo!()}),  _mask(0b0111011, 0b000, 0b0100000)),
    desc(r!(sllw, {todo!()}),  _mask(0b0111011, 0b001, 0b0000000)),
    desc(r!(srllw, {todo!()}), _mask(0b0111011, 0b101, 0b0000000)),
    desc(r!(sraw, {todo!()}),  _mask(0b0111011, 0b101, 0b0100000)),
    
    branch!(beq,  0b000, (|vs1,vs2| vs1==vs2)), // Branch equal
    branch!(bne,  0b001, (|vs1,vs2| vs1!=vs2)), // Branch not equal
    branch!(blt,  0b100, (|vs1,vs2| vs1< vs2)), // Branch less than
    branch!(bge,  0b101, (|vs1,vs2| vs1< vs2)), // Branch less than unsigned
    branch!(bltu, 0b110, (|vs1,vs2| vs1>=vs2)), // Branch greater or equal
    branch!(bgeu, 0b111, (|vs1,vs2| vs1>=vs2)), // Branch greater or equal
    
    desc(i!(jalr, {
        let prev_pc = vm.cpu.pc+4;
        let imm: i16 = unsafe {core::mem::transmute(imm)};
        let (add,overflowed) = vm.cpu.pc.overflowing_add_signed((imm as i64+vs1 as i64)-4);
        if overflowed {todo!();}
        prev_pc
    }),  _mask(0b1100111, 0b000, 0b0)),
    desc(j!(jal, {
        let prev_pc = vm.cpu.pc+4;
        let imm: i16 = unsafe {core::mem::transmute(imm)};
        let (add,overflowed) = vm.cpu.pc.overflowing_add_signed(imm as i64-4);
        if overflowed {todo!();}
        vm.cpu.pc = add;
        prev_pc
    }),   _mask(0b1101111, 0b0, 0b0)),
    
    desc(i!(ecall,  {
        dbg!(imm, rs1, rd);
        match imm {
            0 => {
                todo!("ECALL")
            },
            1 => {
                todo!("EBREAK")
            },
            0x302 => { // MRET
                let mepc = crate::csr!(vm, mepc);
                println!("MRET");
                vm.cpu.pc = mepc.0;
                0
            },
            _ => {todo!("Invalid instruction {:?}", instruction.0)}
        }
    }), _mask(0b1110011, 0b0, 0b0)), // Immediates (fun7)
    desc(i!(ebreak, {todo!()}), _mask(0b1110011, 0b0, 0b1)),
    
    // Atomic Read/Write CSR
    desc(i!(csrrw, {
        let old = *vm.cpu.csr(CsrID::new(imm));
        println!("{}", CsrID::new(imm));
        vm.cpu.csr(CsrID::new(imm)).0 = vs1;
        old.0
    }), _mask(0b1110011, 0b001, 0b0)),
    // Atomic Read and Set Bits in CSR
    desc(i!(csrrs, {
        assert_eq!(rs1, Reg::zero); // Not supported if non-zero
        vm.cpu.csr(CsrID::new(imm)).0
    }), _mask(0b1110011, 0b010, 0b0)),
    // Atomic Read and Clear Bits in CSR
    desc(i!(csrrc, {todo!()}), _mask(0b1110011, 0b011, 0b0)),
    // Same but with immediates
    desc(i!(csrrwi,{todo!()}), _mask(0b1110011, 0b101, 0b0)),
    desc(i!(csrrsi,{todo!()}), _mask(0b1110011, 0b110, 0b0)),
    desc(i!(csrrci,{todo!()}), _mask(0b1110011, 0b111, 0b0)),
];

pub enum InstructionDescription {
    Base(InstructionDescription32),
    Compressed(InstructionDescription16),
}       

pub fn get_from_opcode(opcode:u8) -> Option<&'static Vec<InstructionDescription>> {
    unsafe{REVERSE_INSTRUCTIONS_MASKS.get()?.get(opcode as usize)}
}
pub fn try_find_instruction32_desc(inst: Instruction32) -> Result<InstructionDescription> {
    let opcode = inst.opcode();
    let neighbors = get_from_opcode(opcode).context("Can't find opcode")?;
    if neighbors.is_empty() {return Err(color_eyre::Report::msg(format!("Invalid opcode ({opcode}, {inst:?})")));}
    let fmt = neighbors[0].1;
    for (_name, _fmt, mask, fun) in neighbors {
        let mi = Instruction32(mask.0);
        if match fmt {
            Instruction32Format::R => {mi.fun3() == inst.fun3() && mi.fun7() == inst.fun7()},
            Instruction32Format::I => {mi.fun3() == inst.fun3()},
            Instruction32Format::S => {mi.fun3() == inst.fun3()},
            Instruction32Format::B => {mi.fun3() == inst.fun3()},
            Instruction32Format::U => {true},
            Instruction32Format::J => {true},
        } {
            return Ok((_name, fmt, *mask, *fun))
        }
    }
    Err(Report::msg(format!("Didn't find instruction description: {:b}", inst.0)))
}
pub fn find_instruction32_desc(inst: Instruction32) -> InstructionDescription32 {
    try_find_instruction32_desc(inst).unwrap()
}

type _ReverseInstructionsMasks = [Vec<InstructionDescription32>; 127];
pub static mut REVERSE_INSTRUCTIONS_MASKS: OnceCell<_ReverseInstructionsMasks> = OnceCell::new();
pub fn set_instructions_funcs() {
    let mut instru_funcs: _ReverseInstructionsMasks = std::array::from_fn(|_| Vec::new());
    for (name, format, mask, fun) in INSTRUCTIONS32.iter() {
        let opcode = Instruction32(mask.0).opcode();
        instru_funcs[opcode as usize].push((name, *format, *mask, *fun));
    }
    unsafe{REVERSE_INSTRUCTIONS_MASKS.set(instru_funcs).unwrap()}
}