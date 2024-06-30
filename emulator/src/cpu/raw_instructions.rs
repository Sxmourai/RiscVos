use super::{instructions::Instruction, reg::RegisterValue, CPU};
// Don't forget to edit ./instructions.rs when adding functions

pub fn add(rs1: RegisterValue, rs2: RegisterValue) -> RegisterValue {
    rs1+rs2
}
pub fn addi(rs1: RegisterValue, rs2: u32) -> RegisterValue {
    rs1+rs2
}
pub fn sub(rs1: RegisterValue, rs2: u32) -> RegisterValue {
    rs1-rs2
}