use super::{instructions::Instruction, reg::RegisterValue, CPU};
// Don't forget to edit ./instructions.rs when adding functions
pub type InstructionFunction = fn(&mut crate::vm::VM, RegisterValue, RegisterValue) -> RegisterValue;

pub fn lb       (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    todo!()
}
pub fn lh       (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    todo!()
}
pub fn lw       (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    todo!()
}
pub fn ld       (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    todo!()
}
pub fn lbu      (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    todo!()
}
pub fn lhu      (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    todo!()
}
pub fn lwu      (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    todo!()
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
pub fn slti     (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    todo!()
}
pub fn sltiu    (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    todo!()
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
pub fn sb       (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    todo!()
}
pub fn sh       (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    todo!()
}
pub fn sw       (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    todo!()
}
pub fn sd       (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
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
pub fn lui      (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    todo!()
}
pub fn addw     (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    todo!()
}
pub fn subw     (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    todo!()
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
pub fn beq      (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    todo!()
}
pub fn bne      (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    todo!()
}
pub fn bit      (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    todo!()
}
pub fn bge      (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    todo!()
}
pub fn bitu     (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    todo!()
}
pub fn bgeu     (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    todo!()
}
// Jump and Link Register
pub fn jalr     (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    let rd = vm.cpu.pc+4; // Address of next instruction
    vm.cpu.pc = rs1+rs2-4; // -4 because we add 4 at the end of execution
    rd
}
pub fn jal      (vm: &mut crate::vm::VM, rs1: RegisterValue, rs2: RegisterValue) -> u32 {
    let rd = vm.cpu.pc+4; // Address of next instruction
    vm.cpu.pc = vm.cpu.pc.overflowing_add_signed(rs2 as i32-4).0;
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