use bit_field::BitField;

use crate::{cpu::reg::Reg, uguest};

use super::{iguest, instructions::Instruction, reg::RegValue, CPU};
// Don't forget to edit ./instructions.rs when adding functions
pub type InstructionFunction = fn(&mut crate::vm::VM, RegValue, RegValue) -> RegValue;

macro_rules! todo_instruction {
    ($name:ident) => {
        pub fn $name(vm: &mut crate::vm::VM, rs1: RegValue, rs2: RegValue) -> RegValue {
            todo!()
        }
    };
}

macro_rules! load {
    ($size: ty,$name: ident) => {
        pub fn $name(vm: &mut crate::vm::VM, rs1: RegValue, rs2: RegValue) -> RegValue {
            vm.mem.get::<$size>((rs1+rs2)).unwrap() as _
        }          
    };
}
// pub fn lb(vm: &mut crate::vm::VM, rs1: RegValue, rs2: RegValue) -> u32 {
//     *vm.mem.get::<u8>((rs1+rs2)) as u32
// }
load!(u8, lb);
load!(u16,lh);
load!(u32, lw);
load!(u64, ld);
macro_rules! load_unsigned {
    ($size: ty,$name: ident) => {
        pub fn $name(vm: &mut crate::vm::VM, rs1: RegValue, rs2: RegValue) -> RegValue {
            todo!()
            // *vm.mem.get::<$size>((rs1+rs2)) as _
        }          
    };
}

load_unsigned!(u8, lbu);
load_unsigned!(u16, lhu);
load_unsigned!(u32, lwu);
load_unsigned!(u64, ldu);

todo_instruction!(fence);
todo_instruction!(fencei);

pub fn addi(vm: &mut crate::vm::VM, rs1: RegValue, rs2: RegValue) -> RegValue {
    rs1+rs2
}
pub fn slli(vm: &mut crate::vm::VM, rs1: RegValue, rs2: RegValue) -> RegValue {
    rs1<<rs2
}
// Set less than immediate
pub fn slti(vm: &mut crate::vm::VM, rs1: RegValue, rs2: RegValue) -> RegValue {
    if rs1 < rs2 {1} else {0}
}
pub fn sltiu(vm: &mut crate::vm::VM, rs1: RegValue, rs2: RegValue) -> RegValue {
    if rs1 < rs2 {1} else {0}
}
pub fn xori(vm: &mut crate::vm::VM, rs1: RegValue, rs2: RegValue) -> RegValue {
    rs1^rs2
}
pub fn srli(vm: &mut crate::vm::VM, rs1: RegValue, rs2: RegValue) -> RegValue {
    rs1>>rs2
}
pub fn srai(vm: &mut crate::vm::VM, rs1: RegValue, rs2: RegValue) -> RegValue {
    todo!()
}
pub fn ori(vm: &mut crate::vm::VM, rs1: RegValue, rs2: RegValue) -> RegValue {
    rs1|rs2
}
pub fn andi(vm: &mut crate::vm::VM, rs1: RegValue, rs2: RegValue) -> RegValue {
    rs1&rs2
}
pub fn auipc(vm: &mut crate::vm::VM, rs1: RegValue, rs2: RegValue) -> RegValue {
    vm.cpu.pc+(rs1<<12)
}
pub fn addiw(vm: &mut crate::vm::VM, rs1: RegValue, rs2: RegValue) -> RegValue {
    todo!()
}
pub fn slliw(vm: &mut crate::vm::VM, rs1: RegValue, rs2: RegValue) -> RegValue {
    todo!()
}
pub fn srliw(vm: &mut crate::vm::VM, rs1: RegValue, rs2: RegValue) -> RegValue {
    todo!()
}
pub fn sraiw(vm: &mut crate::vm::VM, rs1: RegValue, rs2: RegValue) -> RegValue {
    todo!()
}

macro_rules! store {
    ($size: ident, $name: ident) => {
        // u64 because we of function signature, but it's a u32 !
        pub fn $name(vm: &mut crate::vm::VM, instruction: u64, _zero: RegValue) -> RegValue {
            let rs2 = *vm.cpu.reg(Reg::new(instruction.get_bits(20..=24) as u8));
            let imm = (instruction.get_bits(7..=11) | (instruction.get_bits(25..=31) << 5));
            let rs1 = vm.cpu.reg(Reg::new(instruction.get_bits(15..=19) as u8));
            vm.mem.set::<$size>((*rs1+imm as uguest), (rs2&($size::MAX as uguest)) as _);
            0
        }
    };
}

store!(u8,  sb);
store!(u16, sh);
store!(u32, sw);
store!(u64, sd);


macro_rules! op {
    ($name: ident, $op: expr) => {
        pub fn $name(vm: &mut crate::vm::VM, rs1: RegValue, rs2: RegValue) -> RegValue {
            $op(rs1,rs2)
        }
    };
}
op!(add, core::ops::Add::add);
op!(sub, core::ops::Sub::sub);
op!(sll, core::ops::Shl::shl);
todo_instruction!(slt);
todo_instruction!(sltu);
op!(xor, core::ops::BitXor::bitxor);
op!(srl, core::ops::Shr::shr);
todo_instruction!(sra); // rs1>>>rs2
op!(or, core::ops::BitOr::bitor);
op!(and, core::ops::BitAnd::bitand);

op!(addw, core::ops::Add::add);
op!(subw, core::ops::Sub::sub);
op!(sllw, core::ops::Shl::shl);
todo_instruction!(srllw);
todo_instruction!(sraw);

macro_rules! branch {
    ($name:ident, $op: expr) => {
        // See $op
        pub fn $name(vm: &mut crate::vm::VM, instruction: u64, _zero: u64) -> u64 {
            let imm = (instruction.get_bits(8..=11)<<1) | (instruction.get_bits(25..=30) << 4) | ((instruction & (1<<7))<<11) | ((instruction & (1<<31))<<12);
            let rs1 = instruction.get_bits(15..=19);
            let rs2 = instruction.get_bits(20..=24);
            if $op(rs1,rs2) {
                vm.cpu.pc += rs2 as uguest
            }
            0
        }
    };
}
branch!(beq,  |rs1,rs2|rs1==rs2); // Branch equal
branch!(bne,  |rs1,rs2|rs1!=rs2); // Branch not equal
branch!(blt,  |rs1,rs2|rs1<rs2);  // Branch less than
branch!(bltu, |rs1,rs2|rs1<rs2);  // Branch less than unsigned
branch!(bge,  |rs1,rs2|rs1>=rs2); // Branch greater or equal
branch!(bgeu, |rs1,rs2|rs1>=rs2); // Branch greater or equal


// Jump and Link Register

pub fn jalr(vm: &mut crate::vm::VM, rs1: RegValue, rs2: RegValue) -> RegValue {
    let pc = &mut vm.cpu.pc;
    let rd = *pc+4; // Address of next instruction
    *pc = rs1+rs2-4; // -4 because we add 4 at the end of execution
    rd
}
pub fn jal(vm: &mut crate::vm::VM, rs1: RegValue, rs2: RegValue) -> RegValue {
    let pc = &mut vm.cpu.pc;
    let rd = *pc+4; // Address of next instruction
    *pc = pc.overflowing_add_signed(rs2 as iguest-4).0;
    rd
}
pub fn ecall(vm: &mut crate::vm::VM, rs1: RegValue, rs2: RegValue) -> RegValue {
    todo!()
}
pub fn ebreak(vm: &mut crate::vm::VM, rs1: RegValue, rs2: RegValue) -> RegValue {
    todo!()
}
pub fn csrrw(vm: &mut crate::vm::VM, rs1: RegValue, rs2: RegValue) -> RegValue {
    todo!()
}
pub fn csrrs(vm: &mut crate::vm::VM, rs1: RegValue, rs2: RegValue) -> RegValue {
    todo!()
}
pub fn csrrc(vm: &mut crate::vm::VM, rs1: RegValue, rs2: RegValue) -> RegValue {
    todo!()
}
pub fn csrrwi(vm: &mut crate::vm::VM, rs1: RegValue, rs2: RegValue) -> RegValue {
    todo!()
}
pub fn csrrsi(vm: &mut crate::vm::VM, rs1: RegValue, rs2: RegValue) -> RegValue {
    todo!()
}
pub fn csrrci(vm: &mut crate::vm::VM, rs1: RegValue, rs2: RegValue) -> RegValue {
    todo!()
}
// U-Format, so the immediate is stored in rs1
pub fn lui(vm: &mut crate::vm::VM, rs1: RegValue, rs2: RegValue) -> RegValue {
    rs1
}
