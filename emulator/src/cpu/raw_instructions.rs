// Big thanks to https://www.eg.bucknell.edu/~csci206/riscv-converter/Annotated_RISCV_Card.pdf
// For more info about instructions https://projectf.io/posts/riscv-cheat-sheet/

use instruction_proc::instruction_r as r;
use instruction_proc::instruction_i as i;
use instruction_proc::instruction_s as s;
use instruction_proc::instruction_u as u;
use instruction_proc::instruction_j as j;
use instruction_proc::instruction_b as b;

pub type Rd = super::reg::Reg;
pub type Rs1 = super::reg::Reg;
pub type Rs2 = super::reg::Reg;
pub type Imm = u16;
pub type UImm = u32;

use crate::uguest;
use crate::cpu::CsrID;

const fn _mask(opcode: u32, fun3: u32, fun7: u32) -> InstructionMask {
    InstructionMask(opcode | fun3 << 12 | fun7 << 25)
}
const fn desc(macro_out: (&'static str, InstructionFormat, InstructionFunction), mask: InstructionMask) -> InstructionDescription {
    (macro_out.0, macro_out.1, mask, macro_out.2)
}

macro_rules! load {
    ($size: ty,$name: ident,$func3: expr) => {
        desc(i!($name, {
            vm.mem.get::<$size>((rs1+imm as uguest)).unwrap() as _
        }), _mask(0b0000011, $func3, 0b0))
        // pub fn $name(vm: &mut crate::vm::VM, instruction: Instruction) {
        //     let (rs1, imm, dest) = Instruction::parse_i(instruction);
        // }
    };
}
macro_rules! store {
    ($size: ty,$name: ident,$func3: expr) => {
        desc(s!($name, {
            vm.mem.set::<$size>((rs1+imm as uguest), (rs2&($size::MAX as uguest)) as _).unwrap();
        }), _mask(0b0100011, $func3, 0b0))
    };
}


macro_rules! op_i {
    ($name: ident, $operator: expr) => {
        i!($name, {
            let res = $operator(rs1, imm as uguest);
            res
        })
    };
}
macro_rules! op_r {
    ($name: ident, $func3: expr, $func7:expr, $operator: expr) => {
        desc(r!($name, {
            let res = $operator(rs1, rs2);
            res
        }), _mask(0b0110011, $func3, $func7))
    };
}

macro_rules! branch {
    ($name: ident, $func3: expr, $op: tt) => {
        desc(s!($name, {
            dbg!(rs1,rs2, imm);
            if $op(rs1,rs2) {
                vm.cpu.pc += rs2
            };
        }), _mask(0b1100011, $func3, 0b0))
    };
}

#[track_caller]
fn panic(a: uguest, b: uguest) -> uguest {
    todo!("{} {}", a,b)
}


#[derive(Clone, Copy, Debug)]
pub struct InstructionMask(pub u32);
#[derive(Debug, Clone, Copy)]
pub enum InstructionFormat {
    R,I,S,B,U,J,
}
pub type InstructionFunction = fn(&mut crate::vm::VM, super::instructions::Instruction);
pub type InstructionDescription = (&'static str, InstructionFormat, InstructionMask, InstructionFunction);
pub const INSTRUCTIONS: [InstructionDescription; 60] = [
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
    
    desc(i!(addiw, {rs1+imm as uguest}), _mask(0b0011011, 0b000, 0b0)),
    desc(i!(slliw, {(rs1)<<(imm as uguest)}), _mask(0b0011011, 0b001, 0b0000000)),
    desc(i!(srliw, {rs1>>imm as uguest}), _mask(0b0011011, 0b101, 0b0000000)),
    desc(i!(sraiw, {todo!("rs1>>>imm as uguest")}), _mask(0b0011011, 0b101, 0b0100000)),
    
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
    
    branch!(beq,  0b000, (|rs1,rs2| rs1==rs2)), // Branch equal
    branch!(bne,  0b001, (|rs1,rs2| rs1!=rs2)), // Branch not equal
    branch!(blt,  0b100, (|rs1,rs2| rs1< rs2)), // Branch less than
    branch!(bge,  0b101, (|rs1,rs2| rs1< rs2)), // Branch less than unsigned
    branch!(bltu, 0b110, (|rs1,rs2| rs1>=rs2)), // Branch greater or equal
    branch!(bgeu, 0b111, (|rs1,rs2| rs1>=rs2)), // Branch greater or equal
    
    desc(i!(jalr, {
        let prev_pc = vm.cpu.pc+4;
        vm.cpu.pc += rs1+imm as uguest;
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
    
    desc(i!(ecall,  {todo!()}), _mask(0b1110011, 0b0, 0b0)), // Immediates (fun7)
    desc(i!(ebreak, {todo!()}), _mask(0b1110011, 0b0, 0b1)),
    
    // Atomic Read/Write CSR
    desc(i!(csrrw, {
        assert_eq!(rs1, 0); // Not supported if non-zero
        dbg!(rs1,imm,rd);
        vm.cpu.csr(CsrID::new(imm)).0 = rs1;
        0 // Write 0 to Reg::zero
    }), _mask(0b1110011, 0b001, 0b0)),
    // Atomic Read and Set Bits in CSR
    desc(i!(csrrs, {
        assert_eq!(rs1, 0); // Not supported if non-zero
        vm.cpu.csr(CsrID::new(imm)).0
    }), _mask(0b1110011, 0b010, 0b0)),
    // Atomic Read and Clear Bits in CSR
    desc(i!(csrrc, {todo!()}), _mask(0b1110011, 0b011, 0b0)),
    // Same but with immediates
    desc(i!(csrrwi,{todo!()}), _mask(0b1110011, 0b101, 0b0)),
    desc(i!(csrrsi,{todo!()}), _mask(0b1110011, 0b110, 0b0)),
    desc(i!(csrrci,{todo!()}), _mask(0b1110011, 0b111, 0b0)),
];


// use bit_field::BitField;

// use crate::{cpu::reg::Reg, uguest};

// use super::{iguest, instructions::Instruction, reg::RegValue, CPU};

// macro_rules! todo_instruction {
//     ($name:ident) => {
//         pub fn $name(vm: &mut crate::vm::VM, instruction: Instruction) {
//             todo!()
//         }
//     };
// }


// load!(u8, lb);
// load!(u16,lh);
// load!(u32, lw);
// load!(u64, ld);
// macro_rules! load_unsigned {
//     ($size: ty,$name: ident) => {
//         todo_instruction!($name);
//     };
// }

// load_unsigned!(u8, lbu);
// load_unsigned!(u16, lhu);
// load_unsigned!(u32, lwu);
// load_unsigned!(u64, ldu);

// todo_instruction!(fence);
// todo_instruction!(fencei);

// instruction_i!(addi, {
//     (rs1+imm as uguest)
// });
// pub fn slli(vm: &mut crate::vm::VM, instruction: Instruction) {
//     rs1<<rs2
// }
// // Set less than immediate
// pub fn slti(vm: &mut crate::vm::VM, instruction: Instruction) {
//     if rs1 < rs2 {1} else {0}
// }
// pub fn sltiu(vm: &mut crate::vm::VM, instruction: Instruction) {
//     if rs1 < rs2 {1} else {0}
// }
// pub fn xori(vm: &mut crate::vm::VM, instruction: Instruction) {
//     rs1^rs2
// }
// pub fn srli(vm: &mut crate::vm::VM, instruction: Instruction) {
//     rs1>>rs2
// }
// pub fn srai(vm: &mut crate::vm::VM, instruction: Instruction) {
//     todo!()
// }
// pub fn ori(vm: &mut crate::vm::VM, instruction: Instruction) {
//     rs1|rs2
// }
// pub fn andi(vm: &mut crate::vm::VM, instruction: Instruction) {
//     rs1&rs2
// }
// pub fn auipc(vm: &mut crate::vm::VM, instruction: Instruction) {
//     vm.cpu.pc+(rs1<<12)
// }
// pub fn addiw(vm: &mut crate::vm::VM, instruction: Instruction) {
//     todo!()
// }
// pub fn slliw(vm: &mut crate::vm::VM, instruction: Instruction) {
//     todo!()
// }
// pub fn srliw(vm: &mut crate::vm::VM, instruction: Instruction) {
//     todo!()
// }
// pub fn sraiw(vm: &mut crate::vm::VM, instruction: Instruction) {
//     todo!()
// }

// // macro_rules! store {
// //     ($size: ident, $name: ident) => {
// //         // u64 because we of function signature, but it's a u32 !
// //         pub fn $name(vm: &mut crate::vm::VM, instruction: u64, _zero: RegValue) -> RegValue {
// //             let rs2 = *vm.cpu.reg(Reg::new(instruction.get_bits(20..=24) as u8));
// //             let imm = (instruction.get_bits(7..=11) | (instruction.get_bits(25..=31) << 5));
// //             let rs1 = vm.cpu.reg(Reg::new(instruction.get_bits(15..=19) as u8));
// //             vm.mem.set::<$size>((rs1+imm as uguest), (rs2&($size::MAX as uguest)) as _);
// //             0
// //         }
// //     };
// // }

// // store!(u8,  sb);
// // store!(u16, sh);
// // store!(u32, sw);
// // store!(u64, sd);


// macro_rules! op {
//     ($name: ident, $op: expr) => {
//         pub fn $name(vm: &mut crate::vm::VM, instruction: Instruction) {
//             $op(rs1,rs2)
//         }
//     };
// }
// op!(add, core::ops::Add::add);
// op!(sub, core::ops::Sub::sub);
// op!(sll, core::ops::Shl::shl);
// todo_instruction!(slt);
// todo_instruction!(sltu);
// op!(xor, core::ops::BitXor::bitxor);
// op!(srl, core::ops::Shr::shr);
// todo_instruction!(sra); // rs1>>>rs2
// op!(or, core::ops::BitOr::bitor);
// op!(and, core::ops::BitAnd::bitand);

// op!(addw, core::ops::Add::add);
// op!(subw, core::ops::Sub::sub);
// op!(sllw, core::ops::Shl::shl);
// todo_instruction!(srllw);
// todo_instruction!(sraw);

// // macro_rules! branch {
// //     ($name:ident, $op: expr) => {
// //         // See $op
// //         pub fn $name(vm: &mut crate::vm::VM, instruction: u32) {
// //             let imm = (instruction.get_bits(8..=11)<<1) | (instruction.get_bits(25..=30) << 4) | ((instruction & (1<<7))<<11) | ((instruction & (1<<31))<<12);
// //             let rs1 = instruction.get_bits(15..=19);
// //             let rs2 = instruction.get_bits(20..=24);
// //             if $op(rs1,rs2) {
// //                 vm.cpu.pc += rs2
// //             }
// //             0
// //         }
// //     };
// // }
// branch!(beq,  |rs1,rs2|rs1==rs2); // Branch equal
// branch!(bne,  |rs1,rs2|rs1!=rs2); // Branch not equal
// branch!(blt,  |rs1,rs2|rs1<rs2);  // Branch less than
// branch!(bltu, |rs1,rs2|rs1<rs2);  // Branch less than unsigned
// branch!(bge,  |rs1,rs2|rs1>=rs2); // Branch greater or equal
// branch!(bgeu, |rs1,rs2|rs1>=rs2); // Branch greater or equal


// // Jump and Link Register

// pub fn jalr(vm: &mut crate::vm::VM, instruction: Instruction) {
//     let pc = &mut vm.cpu.pc;
//     let rd = *pc+4; // Address of next instruction
//     *pc = rs1+rs2-4; // -4 because we add 4 at the end of execution
//     rd
// }
// pub fn jal(vm: &mut crate::vm::VM, instruction: Instruction) {
//     let pc = &mut vm.cpu.pc;
//     let rd = *pc+4; // Address of next instruction
//     *pc = pc.overflowing_add_signed(rs2 as iguest-4).0;
//     rd
// }
// pub fn ecall(vm: &mut crate::vm::VM, instruction: Instruction) {
//     todo!()
// }
// pub fn ebreak(vm: &mut crate::vm::VM, instruction: Instruction) {
//     todo!()
// }
// pub fn csrrw(vm: &mut crate::vm::VM, instruction: u64, csr: RegValue) -> RegValue {
//     let inst = Instruction(instruction as _);
//     dbg!(inst,csr);
//     todo!()
// }
// pub fn csrrs(vm: &mut crate::vm::VM, instruction: u64, csr: RegValue) -> RegValue {
//     let inst = Instruction(instruction as _);
//     if inst.rs1() != Reg::zero {todo!()}
//     vm.cpu.csrs[csr as usize].0
// }
// pub fn csrrc(vm: &mut crate::vm::VM, instruction: u64, csr: RegValue) -> RegValue {
//     let inst = Instruction(instruction as _);
//     todo!()
// }
// pub fn csrrwi(vm: &mut crate::vm::VM, instruction: u64, csr: RegValue) -> RegValue {
//     let inst = Instruction(instruction as _);
//     todo!()
// }
// pub fn csrrsi(vm: &mut crate::vm::VM, instruction: u64, csr: RegValue) -> RegValue {
//     let inst = Instruction(instruction as _);
//     todo!()
// }
// pub fn csrrci(vm: &mut crate::vm::VM, instruction: u64, csr: RegValue) -> RegValue {
//     let inst = Instruction(instruction as _);
//     todo!()
// }
// // U-Format, so the immediate is stored in rs1
// pub fn lui(vm: &mut crate::vm::VM, instruction: Instruction) {
//     rs1
// }
