use bit_field::BitField;

use super::{instructions::Instruction, reg::RegisterValue, CPU};
// Don't forget to edit ./instructions.rs when adding functions
pub type InstructionFunction = fn(&mut crate::vm::VM, RegisterValue, RegisterValue) -> RegisterValue;

pub fn lb       (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    vm.memory[(rs1+rs2) as usize] as _
}
pub fn lh       (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    vm.get_T::<u16>((rs1+rs2) as usize/2) as _
}
pub fn lw       (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    vm.get_dword((rs1+rs2) as usize/4)
}
pub fn ld       (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    todo!()
}
// Load byte Unsigned
pub fn lbu      (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    vm.memory[(rs1+rs2) as usize] as _
}
pub fn lhu      (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    vm.get_T::<u16>((rs1+rs2) as usize/2) as _
}
pub fn lwu      (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    vm.get_dword((rs1+rs2) as usize/4)
}
pub fn fence    (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    todo!()
}
pub fn fencei   (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    todo!()
}
pub fn addi     (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    rs1+rs2
}
pub fn slli     (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    rs1<<rs2
}
// Set less than immediate
pub fn slti     (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    if rs1 < rs2 {1} else {0}
}
pub fn sltiu    (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    if rs1 < rs2 {1} else {0}
}
pub fn xori     (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    rs1^rs2
}
pub fn srli     (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    rs1>>rs2
}
pub fn srai     (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    todo!()
}
pub fn ori      (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    rs1|rs2
}
pub fn andi     (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    rs1&rs2
}
pub fn auipc    (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    vm.cpu.pc+(rs1<<12)
}
pub fn addiw    (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    todo!()
}
pub fn slliw    (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    todo!()
}
pub fn srliw    (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    todo!()
}
pub fn sraiw    (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    todo!()
}

pub fn _store<T: Copy>(vm: &mut crate::vm::VM, instruction: u32, rs2: T) -> u32 {
    let imm = (instruction.get_bits(7..=11) | (instruction.get_bits(25..=31) << 5));
    let rs1 = vm.cpu.as_array()[instruction.get_bits(15..=19) as usize];
    dbg!(imm, rs1, instruction);

    vm.set_T::<T>(((rs1+imm) as usize)/core::mem::size_of::<T>(), rs2);
    0
}

pub fn sb       (vm: &mut crate::vm::VM, instruction: u32, _zero: RegisterValue) -> u32 {
    let rs2 = vm.cpu.as_array()[instruction.get_bits(20..=24) as usize];
    _store::<u8>(vm, instruction, (rs2&u8::MAX as u32) as u8)
}
pub fn sh       (vm: &mut crate::vm::VM, instruction: u32, _zero: RegisterValue) -> u32 {
    let rs2 = vm.cpu.as_array()[instruction.get_bits(20..=24) as usize];
    _store::<u16>(vm, instruction, (rs2&u16::MAX as u32) as u16)
}
pub fn sw       (vm: &mut crate::vm::VM, instruction: u32, _zero: RegisterValue) -> u32 {
    let rs2 = vm.cpu.as_array()[instruction.get_bits(20..=24) as usize];
    _store::<u32>(vm, instruction, rs2)
}
pub fn sd       (vm: &mut crate::vm::VM, instruction: u32, _zero: RegisterValue) -> u32 {
    todo!()
}
pub fn add      (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    rs1+rs2
}
pub fn sub      (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    rs1-rs2
}
pub fn sll      (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    rs1<<rs2
}
pub fn slt      (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    todo!()
}
pub fn sltu     (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    todo!()
}
pub fn xor      (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    rs1^rs2
}
pub fn srl      (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    rs1>>rs2
}
pub fn sra      (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    todo!()
    // rs1>>>rs2
}
pub fn or       (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    rs1|rs2
}
pub fn and      (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    rs1&rs2
}
// U-Format, so the immediate is stored in rs1
pub fn lui      (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    rs1
}
pub fn addw     (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    rs1+rs2
}
pub fn subw     (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    rs1-rs2
}
pub fn sllw     (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    todo!()
}
pub fn srllw    (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    todo!()
}
pub fn sraw     (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    todo!()
}
fn _branch(vm:&mut crate::vm::VM, instruction: u32, op: fn(u32, u32) -> bool) -> u32 {
    let imm = (instruction.get_bits(8..=11)<<1) | (instruction.get_bits(25..=30) << 4) | ((instruction & (1<<7))<<11) | ((instruction & (1<<31))<<12);
    let rs1 = instruction.get_bits(15..=19);
    let rs2 = instruction.get_bits(20..=24);
    let pc = &mut vm.cpu.pc;
    if op(rs1,rs2) {
        *pc += rs2
    }
    0
}

// Branch equal
pub fn beq      (vm: &mut crate::vm::VM, instruction: u32, _zero: u32) -> u32 {
    _branch(vm, instruction, |rs1,rs2| rs1==rs2)
}
// Branch not equal
pub fn bne      (vm: &mut crate::vm::VM, instruction: u32, _zero: u32) -> u32 {
    _branch(vm, instruction, |rs1,rs2| rs1!=rs2)
}
// Branch less than
pub fn blt      (vm: &mut crate::vm::VM, instruction: u32, _zero: u32) -> u32 {
    _branch(vm, instruction, |rs1,rs2| rs1<rs2)
}
// Branch Greater or Equal
pub fn bge      (vm: &mut crate::vm::VM, instruction: u32, _zero: u32) -> u32 {
    _branch(vm, instruction, |rs1,rs2| rs1>=rs2)
}
// Branch Less Than Unsigned
pub fn bltu     (vm: &mut crate::vm::VM, instruction: u32, _zero: u32) -> u32 {
    _branch(vm, instruction, |rs1,rs2| rs1<rs2)
}
// Branch Greater or Equal Unsigned
pub fn bgeu     (vm: &mut crate::vm::VM, instruction: u32, _zero: u32) -> u32 {
    _branch(vm, instruction, |rs1,rs2| rs1>=rs2)
}
// Jump and Link Register
pub fn jalr     (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    let pc = &mut vm.cpu.pc;
    let rd = *pc+4; // Address of next instruction
    *pc = rs1+rs2-4; // -4 because we add 4 at the end of execution
    rd
}
pub fn jal      (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    let pc = &mut vm.cpu.pc;
    let rd = *pc+4; // Address of next instruction
    *pc = pc.overflowing_add_signed(rs2 as i32-4).0;
    rd
}
pub fn ecall    (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    todo!()
}
pub fn ebreak   (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    todo!()
}
pub fn CSRRW    (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    todo!()
}
pub fn CSRRS    (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    todo!()
}
pub fn CSRRC    (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    todo!()
}
pub fn CSRRWI   (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    todo!()
}
pub fn CSRRSI   (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    todo!()
}
pub fn CSRRCI   (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    todo!()
}