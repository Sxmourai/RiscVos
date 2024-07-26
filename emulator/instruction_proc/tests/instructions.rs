
pub type Rd = u8;
pub type Rs1 = u8;
pub type Rs2 = u8;
pub type Imm = u16;
pub type UImm = u32;

mod vm {
    pub struct VM();
}
use instruction_proc::*;
#[test]
fn test_instruction_i() {
    instruction_i!(csrrs, {
        todo!();
    });
}