use color_eyre::{eyre::ContextCompat, Report, Result};
use std::cell::OnceCell;
use bit_field::BitField;
use super::{raw_instructions::*, reg::Reg, CPU};



pub enum Instruction {
    Base(Instruction32),
    Compressed(Instruction16),
}
impl Instruction {
    /// Returns a Instruction32 or two compressed 16's instructions
    pub fn new(instruction: u32) -> Result<(Self, Option<Instruction16>)> {
        if instruction & 0b11 == 0b11 {
            return Ok((Self::Base(Instruction32::new(instruction)?), None))
        } else {
            return Ok((Self::Compressed(Instruction16::new((instruction & 0xFFFF) as _)), Some(Instruction16::new((instruction>>16) as _))))
        }
    }
}

#[derive(Clone, Copy)]
pub struct Instruction32(pub u32);
impl Instruction32 {
    pub fn new(inst: u32) -> Result<Self> {
        let s = Self(inst);
        try_find_instruction32_desc(s)?;
        Ok(s)
    }
    pub fn parse_r(self) -> (Rs1, Rs2, Rd) {
        (self.rs1() as _,self.rs2() as _, self.rd())
    }
    pub fn parse_i(self) -> (Imm, Rs1, Rd) {
        (self.0.get_bits(20..=31) as _,self.rs1() as _,self.rd())
    }
    // Output in Imm
    pub fn parse_s(self) -> (Imm, Rs1, Rs2) {
        ((self.0.get_bits(7..=11)|(self.0.get_bits(25..=31)<<5)) as _,self.rs1() as _,self.rs2() as _)
    }
    pub fn parse_b(self) -> (Imm, Rs1, Rs2) {
        (((self.0.get_bits(8..=11)<<1) | (self.0.get_bits(25..=30) << 4) | ((self.0 & (1<<7))<<11) | ((self.0 & (1<<31))<<12)) as _,
        self.rs1() as _,self.rs2() as _)
    }
    pub fn parse_u(self) -> (UImm, Rd) { // 19 bits Imm
        ((self.0 & 0xFFFFF000) as _,
        self.rd() as _,)
    }
    pub fn parse_j(self) -> (Imm, Rd) {
        (((self.0.get_bits(21..=30)<<1) | (self.0.get_bits(20..=20)<<11) | (self.0 & 0x7F000) | (self.0.get_bits(31..=31)<<20)) as _,
        self.rd() as _,)
    }

    // 7 bits
    pub fn opcode(self) -> u8 {
        self.0.get_bits(0..=6) as _ // Unwrap unchecked
    }
    // Only usefull if instruction is of type R, I, U, J
    // Destination register
    // 4 bits
    pub fn rd(self) -> Reg {
        Reg::new(self._raw_rd())
    }
    pub fn _raw_rd(self) -> u8 {
        self.0.get_bits(7..=11) as _ // Unwrap unchecked
    }
    // 4 bits
    // Register source 1
    pub fn rs1(self) -> Reg {
        Reg::new(self._raw_rs1())
    }
    pub fn _raw_rs1(self) -> u8 {
        self.0.get_bits(15..=19) as _ // Unwrap unchecked
    }
    // 4 bits
    // Register source 2
    pub fn rs2(self) -> Reg {
        Reg::new(self._raw_rs2())
    }
    pub fn _raw_rs2(self) -> u8 {
        self.0.get_bits(20..=24) as _ // Unwrap unchecked
    }
    // 2 bits (more info about operation)
    pub fn fun3(self) -> u32 {
        self.0.get_bits(12..=14)
    }
    // 6 bits (more info about operation)
    pub fn fun7(self) -> u32 {
        self.0.get_bits(25..=31)
    }
    pub fn format(self) -> Instruction32Format {
        find_instruction32_desc(self).1
    }
    // Depends on format (see self.format and `InstructionFormat`)
    pub fn auto_imm(self) -> u32 {
        match self.format() {
            Instruction32Format::R => {panic!("No immediate in R format !")},
            Instruction32Format::I => {self.0.get_bits(25..=31)},
            Instruction32Format::S => {self.0.get_bits(7..=11) | (self.0.get_bits(25..=31) << 5)},
            Instruction32Format::B => {(self.0.get_bits(8..=11)<<1) | (self.0.get_bits(25..=30) << 4) | ((self.0 & (1<<7))<<11) | ((self.0 & (1<<31))<<12)},
            Instruction32Format::U => {self.0 & 0xFFFFF000},
            Instruction32Format::J => {(self.0.get_bits(21..=30)<<1) | (self.0.get_bits(20..=20)<<11) | (self.0 & 0x7F000) | (self.0.get_bits(31..=31)<<20)},
        }
    }
    fn _opcode_name(self) -> &'static str {
        find_instruction32_desc(self).0
        // println!("Unknown instruction: {:x}", self.0);
        // "unknown" // Could return option ?
    }

    pub fn destination(self) -> Destination {
        match self.format() {
            Instruction32Format::R => Destination::CpuRegister(self.rd()),
            Instruction32Format::I => Destination::CpuRegister(self.rd()),
            Instruction32Format::S => Destination::CpuRegister(Reg::zero),
            Instruction32Format::B => Destination::CpuRegister(Reg::zero),
            Instruction32Format::U => Destination::CpuRegister(self.rd()),
            Instruction32Format::J => Destination::CpuRegister(self.rd()),
        }
    }
    // Returns the first input and tells if there is a second input (see `self.s2`)
    pub fn s1(self) -> (Destination, bool) {
        match self.format() {
            Instruction32Format::R => (Destination::CpuRegister(self.rs1()), true),
            Instruction32Format::I => (Destination::Immediate(self.0), true),
            Instruction32Format::S => (Destination::Immediate(self.0), true), // S has 3 inputs and no outputs, so we put everything in one number
            Instruction32Format::B => (Destination::Immediate(self.0), false), // B has 3 inputs and no outputs, so we put everything in one number
            Instruction32Format::U => (Destination::Immediate(self.0 & 0xFFFFF000), false),
            Instruction32Format::J => (Destination::Immediate((self.0.get_bits(21..=30)<<1) | (self.0.get_bits(20..=20)<<11) | (self.0.get_bits(12..=19)<<12) | (self.0.get_bits(31..=31)<<20)), false),
        }
    }
    pub fn s2(self) -> Destination {
        match self.format() {
            Instruction32Format::R => Destination::CpuRegister(self.rs2()),
            Instruction32Format::I => Destination::Immediate(self.0.get_bits(20..=31)),
            Instruction32Format::S => {Destination::CpuRegister(self.rs2())},
            Instruction32Format::B => {println!("WARN: Trying to get s2 of a B format");Destination::Immediate(0)},
            Instruction32Format::U => {println!("WARN: Trying to get s2 of a U format");Destination::Immediate(0)}, // No rs2
            Instruction32Format::J => {println!("WARN: Trying to get s2 of a J format");Destination::Immediate(0)}, // No rs2
        }
    }
}
impl std::fmt::Debug for Instruction32 {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { 
        fmt.write_str(&format!("{:b} {:b} {:b} {:b}", self.opcode(), self.fun3(), self.fun7(), self.0))
    }
}
impl std::fmt::Display for Instruction32 {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let (s1, has_s2) = self.s1();
        let s2 = if has_s2 {self.s2()} else {Destination::Immediate(0)};
        fmt.write_str(&format!("{} {} {} {}", self._opcode_name(), self.destination(), s1, s2))
    }
}


#[derive(Clone, Copy)]
pub struct Instruction16(pub u16);
impl Instruction16 {
    pub fn new(instruction: u16) -> Self {
        let _s = Self(instruction);
        
        _s
    }
    pub fn opcode(self) -> u8 {
        (self.0 & 0b11) as _
    }
}








#[derive(Debug)]
pub enum Destination {
    CpuRegister(Reg),
    Immediate(u32),
}

impl std::fmt::Display for Destination {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = match self {
            Destination::CpuRegister(reg) => format!("{:?}", reg),
            Destination::Immediate(imm) => format!("{}", imm),
        };
        f.write_fmt(format_args!("{}", val))
    }
}




// impl InstructionMask {
//     // Returns true if the mask corresponds to the same instruction
//     // Like if opcode is same, fun3 and fun7 if there is one
//     pub fn is_mask(self, instruction: Instruction32) -> bool {
//         let s = Instruction32(self.0);
//         if instruction.opcode() == s.opcode() {
//             return match instruction.format() {
//                 InstructionFormat::R => {s.fun3() == instruction.fun3() && s.fun7() == instruction.fun7()},
//                 InstructionFormat::I => {s.fun3() == instruction.fun3()},
//                 InstructionFormat::S => {s.fun3() == instruction.fun3()},
//                 InstructionFormat::B => {s.fun3() == instruction.fun3()},
//                 InstructionFormat::U => {true},
//                 InstructionFormat::J => {true},
//             }
//         }
//         false
//     }
// }


// Not used but can be usefull for documentation

// bitfield::bitfield! {
//     pub struct RInstruction(u32);
//     impl Debug;
//     pub opcode, _: 6, 0;
//     pub rd, _: 11, 7;
//     pub fun3, _: 14, 12;
//     pub rs1, _: 19, 15;
//     pub rs2, _: 24, 20;
//     pub func7, _: 31, 25;
// }
// bitfield::bitfield! {
//     pub struct IInstruction(u32);
//     impl Debug;
//     pub opcode, _: 6, 0;
//     pub rd, _: 11, 7;
//     pub fun3, _: 14, 12;
//     pub rs1, _: 19, 15;
//     pub imm, _: 31, 20;
// }
// bitfield::bitfield! {
//     pub struct SInstruction(u32);
//     impl Debug;
//     pub opcode, _: 6, 0;
//     pub lo_imm, _: 11, 7;
//     pub fun3, _: 14, 12;
//     pub rs1, _: 19, 15;
//     pub rs2, _: 24, 20;
//     pub hi_imm, _: 31, 25;
// }
// bitfield::bitfield! {
//     pub struct BInstruction(u32);
//     impl Debug;
//     pub opcode, _: 6, 0;
//     pub lo_imm, _: 11, 7;
//     pub fun3, _: 14, 12;
//     pub rs1, _: 19, 15;
//     pub rs2, _: 24, 20;
//     pub hi_imm, _: 31, 25;
// }