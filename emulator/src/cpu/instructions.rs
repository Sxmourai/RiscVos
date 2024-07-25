use std::cell::OnceCell;
use bit_field::BitField;
use super::reg::{Register, RegisterValue};
use super::{raw_instructions::*, CPU};

const fn _mask(opcode: u32, fun3: u32, fun7: u32) -> InstructionMask {
    InstructionMask(opcode | fun3 << 12 | fun7 << 25)
}

// Big thanks to https://www.eg.bucknell.edu/~csci206/riscv-converter/Annotated_RISCV_Card.pdf
// For more info about instructions https://projectf.io/posts/riscv-cheat-sheet/
pub const INSTRUCTIONS_MASKS: [(&'static str, InstructionFormat, InstructionMask, InstructionFunction); 59] = [
    ("lb",      InstructionFormat::I, _mask(0b0000011, 0b000, 0b0), crate::cpu::raw_instructions::lb), // ! Do we need to set fun7 ?
    ("lh",      InstructionFormat::I, _mask(0b0000011, 0b001, 0b0), crate::cpu::raw_instructions::lh),
    ("lw",      InstructionFormat::I, _mask(0b0000011, 0b010, 0b0), crate::cpu::raw_instructions::lw),
    ("ld",      InstructionFormat::I, _mask(0b0000011, 0b011, 0b0), crate::cpu::raw_instructions::ld),
    ("lbu",     InstructionFormat::I, _mask(0b0000011, 0b100, 0b0), crate::cpu::raw_instructions::lbu),
    ("lhu",     InstructionFormat::I, _mask(0b0000011, 0b110, 0b0), crate::cpu::raw_instructions::lhu),
    ("lwu",     InstructionFormat::I, _mask(0b0000011, 0b111, 0b0), crate::cpu::raw_instructions::lwu),
    
    ("fence",   InstructionFormat::I, _mask(0b0001111, 0b000, 0b0), crate::cpu::raw_instructions::fence),
    ("fence.i", InstructionFormat::I, _mask(0b0001111, 0b001, 0b0), crate::cpu::raw_instructions::fencei),
    
    ("addi",    InstructionFormat::I, _mask(0b0010011, 0b000, 0b0), crate::cpu::raw_instructions::addi),
    ("slli",    InstructionFormat::I, _mask(0b0010011, 0b001, 0b0), crate::cpu::raw_instructions::slli), // Has funct7 ??
    ("slti",    InstructionFormat::I, _mask(0b0010011, 0b010, 0b0), crate::cpu::raw_instructions::slti),
    ("sltiu",   InstructionFormat::I, _mask(0b0010011, 0b011, 0b0), crate::cpu::raw_instructions::sltiu),
    ("xori",    InstructionFormat::I, _mask(0b0010011, 0b100, 0b0), crate::cpu::raw_instructions::xori),
    ("srli",    InstructionFormat::I, _mask(0b0010011, 0b101, 0b0000000), crate::cpu::raw_instructions::srli),
    ("srai",    InstructionFormat::I, _mask(0b0010011, 0b101, 0b0100000), crate::cpu::raw_instructions::srai),
    ("ori",     InstructionFormat::I, _mask(0b0010011, 0b110, 0b0), crate::cpu::raw_instructions::ori),
    ("andi",    InstructionFormat::I, _mask(0b0010011, 0b111, 0b0), crate::cpu::raw_instructions::andi),
    
    ("auipc",   InstructionFormat::U, _mask(0b0010111, 0b000, 0b0), crate::cpu::raw_instructions::auipc),
    
    ("addiw",   InstructionFormat::I, _mask(0b0011011, 0b000, 0b0), crate::cpu::raw_instructions::addiw),
    ("slliw",   InstructionFormat::I, _mask(0b0011011, 0b001, 0b0000000), crate::cpu::raw_instructions::slliw),
    ("srliw",   InstructionFormat::I, _mask(0b0011011, 0b101, 0b0000000), crate::cpu::raw_instructions::srliw),
    ("sraiw",   InstructionFormat::I, _mask(0b0011011, 0b101, 0b0100000), crate::cpu::raw_instructions::sraiw),

    ("sb",   InstructionFormat::S, _mask(0b0100011, 0b000, 0b0), crate::cpu::raw_instructions::sb),
    ("sh",   InstructionFormat::S, _mask(0b0100011, 0b001, 0b0), crate::cpu::raw_instructions::sh),
    ("sw",   InstructionFormat::S, _mask(0b0100011, 0b010, 0b0), crate::cpu::raw_instructions::sw),
    ("sd",   InstructionFormat::S, _mask(0b0100011, 0b011, 0b0), crate::cpu::raw_instructions::sd),

    ("add",  InstructionFormat::R, _mask(0b0110011, 0b000, 0b0000000), crate::cpu::raw_instructions::add),
    ("sub",  InstructionFormat::R, _mask(0b0110011, 0b000, 0b0100000), crate::cpu::raw_instructions::sub),
    ("sll",  InstructionFormat::R, _mask(0b0110011, 0b001, 0b0000000), crate::cpu::raw_instructions::sll),
    ("slt",  InstructionFormat::R, _mask(0b0110011, 0b010, 0b0000000), crate::cpu::raw_instructions::slt),
    ("sltu", InstructionFormat::R, _mask(0b0110011, 0b011, 0b0000000), crate::cpu::raw_instructions::sltu),
    ("xor",  InstructionFormat::R, _mask(0b0110011, 0b100, 0b0000000), crate::cpu::raw_instructions::xor),
    ("srl",  InstructionFormat::R, _mask(0b0110011, 0b101, 0b0000000), crate::cpu::raw_instructions::srl),
    ("sra",  InstructionFormat::R, _mask(0b0110011, 0b101, 0b0100000), crate::cpu::raw_instructions::sra),
    ("or",   InstructionFormat::R, _mask(0b0110011, 0b110, 0b0000000), crate::cpu::raw_instructions::or),
    ("and",  InstructionFormat::R, _mask(0b0110011, 0b111, 0b0000000), crate::cpu::raw_instructions::and),

    ("lui",  InstructionFormat::U, _mask(0b0110111, 0b0, 0b0), crate::cpu::raw_instructions::lui),
    
    ("addw", InstructionFormat::R, _mask(0b0111011, 0b000, 0b0000000), crate::cpu::raw_instructions::addw),
    ("subw", InstructionFormat::R, _mask(0b0111011, 0b000, 0b0100000), crate::cpu::raw_instructions::subw),
    ("sllw", InstructionFormat::R, _mask(0b0111011, 0b001, 0b0000000), crate::cpu::raw_instructions::sllw),
    ("srllw",InstructionFormat::R, _mask(0b0111011, 0b101, 0b0000000), crate::cpu::raw_instructions::srllw),
    ("sraw", InstructionFormat::R, _mask(0b0111011, 0b101, 0b0100000), crate::cpu::raw_instructions::sraw),

    ("beq",  InstructionFormat::B, _mask(0b1100011, 0b000, 0b0), crate::cpu::raw_instructions::beq),
    ("bne",  InstructionFormat::B, _mask(0b1100011, 0b001, 0b0), crate::cpu::raw_instructions::bne),
    ("blt",  InstructionFormat::B, _mask(0b1100011, 0b100, 0b0), crate::cpu::raw_instructions::blt),
    ("bge",  InstructionFormat::B, _mask(0b1100011, 0b101, 0b0), crate::cpu::raw_instructions::bge),
    ("bltu", InstructionFormat::B, _mask(0b1100011, 0b110, 0b0), crate::cpu::raw_instructions::bltu),
    ("bgeu", InstructionFormat::B, _mask(0b1100011, 0b111, 0b0), crate::cpu::raw_instructions::bgeu),

    ("jalr", InstructionFormat::I, _mask(0b1100111, 0b000, 0b0), crate::cpu::raw_instructions::jalr),
    ("jal",  InstructionFormat::U, _mask(0b1101111, 0b0, 0b0), crate::cpu::raw_instructions::jal),

    ("ecall",  InstructionFormat::I, _mask(0b1110011, 0b0, 0b0), crate::cpu::raw_instructions::ecall), // Immediates (fun7)
    ("ebreak", InstructionFormat::I, _mask(0b1110011, 0b0, 0b1), crate::cpu::raw_instructions::ebreak),

    ("csrrw",  InstructionFormat::I, _mask(0b1110011, 0b001, 0b0), crate::cpu::raw_instructions::csrrw),
    ("csrrs",  InstructionFormat::I, _mask(0b1110011, 0b010, 0b0), crate::cpu::raw_instructions::csrrs),
    ("csrrc",  InstructionFormat::I, _mask(0b1110011, 0b011, 0b0), crate::cpu::raw_instructions::csrrc),
    ("csrrwi", InstructionFormat::I, _mask(0b1110011, 0b101, 0b0), crate::cpu::raw_instructions::csrrwi),
    ("csrrsi", InstructionFormat::I, _mask(0b1110011, 0b110, 0b0), crate::cpu::raw_instructions::csrrsi),
    ("csrrci", InstructionFormat::I, _mask(0b1110011, 0b111, 0b0), crate::cpu::raw_instructions::csrrci),
];



fn get_from_opcode(opcode:u32) -> &'static Vec<InstructionDescription> {
    unsafe{&REVERSE_INSTRUCTIONS_MASKS.get().unwrap()[opcode as usize]}
}
pub fn try_find_instruction_desc(inst: Instruction) -> Option<InstructionDescription> {
    let opcode = inst.opcode();
    let neighbors = get_from_opcode(opcode);
    if neighbors.is_empty() {return None;}
    let fmt = neighbors[0].1;
    for (_name, _fmt, mask, fun) in neighbors {
        let mi = Instruction(mask.0);
        if match fmt {
            InstructionFormat::R => {mi.fun3() == inst.fun3() && mi.fun7() == inst.fun7()},
            InstructionFormat::I => {mi.fun3() == inst.fun3()},
            InstructionFormat::S => {mi.fun3() == inst.fun3()},
            InstructionFormat::B => {mi.fun3() == inst.fun3()},
            InstructionFormat::U => {true},
            InstructionFormat::J => {true},
        } {
            return Some((_name, fmt, *mask, *fun))
        }
    }
    println!("Didn't find instruction description: {:b}", inst.0);
    None
}
pub fn find_instruction_desc(inst: Instruction) -> InstructionDescription {
    try_find_instruction_desc(inst).unwrap()
}

type InstructionDescription = (&'static str, InstructionFormat, InstructionMask, InstructionFunction);
type _ReverseInstructionsMasks = [Vec<InstructionDescription>; 127];
pub static mut REVERSE_INSTRUCTIONS_MASKS: OnceCell<_ReverseInstructionsMasks> = OnceCell::new();
pub fn set_instructions_funcs() {
    let mut instru_funcs: _ReverseInstructionsMasks = std::array::from_fn(|_| Vec::new());
    for (name, format, mask, fun) in INSTRUCTIONS_MASKS.iter() {
        let opcode = Instruction(mask.0).opcode();
        instru_funcs[opcode as usize].push((name, *format, *mask, *fun));
    }
    unsafe{REVERSE_INSTRUCTIONS_MASKS.set(instru_funcs).unwrap()}
}


#[derive(Clone, Copy)]
pub struct Instruction(pub u32);
impl Instruction {
    pub fn new(inst: u32) -> Option<Self> {
        let s = Self(inst);
        try_find_instruction_desc(s)?;
        Some(s)
    }
    // 7 bits
    pub fn opcode(self) -> u32 {
        self.0.get_bits(0..=6)
    }
    // Only usefull if instruction is of type R, I, U, J
    // Destination register
    // 4 bits
    pub fn rd(self) -> Register {
        Register::new(self._raw_rd())
    }
    pub fn _raw_rd(self) -> u32 {
        self.0.get_bits(7..=11)
    }
    // 4 bits
    // Register source 1
    pub fn rs1(self) -> Register {
        Register::new(self._raw_rs1())
    }
    pub fn _raw_rs1(self) -> u32 {
        self.0.get_bits(15..=19)
    }
    // 4 bits
    // Register source 2
    pub fn rs2(self) -> Register {
        Register::new(self._raw_rs2())
    }
    pub fn _raw_rs2(self) -> u32 {
        self.0.get_bits(20..=24)
    }
    // 2 bits (more info about operation)
    pub fn fun3(self) -> u32 {
        self.0.get_bits(12..=14)
    }
    // 6 bits (more info about operation)
    pub fn fun7(self) -> u32 {
        self.0.get_bits(25..=31)
    }
    pub fn format(self) -> InstructionFormat {
        find_instruction_desc(self).1
    }
    // Depends on format (see self.format and `InstructionFormat`)
    pub fn auto_imm(self) -> u32 {
        match self.format() {
            InstructionFormat::R => {panic!("No immediate in R format !")},
            InstructionFormat::I => {self.0.get_bits(25..=31)},
            InstructionFormat::S => {self.0.get_bits(7..=11) | (self.0.get_bits(25..=31) << 5)},
            InstructionFormat::B => {(self.0.get_bits(8..=11)<<1) | (self.0.get_bits(25..=30) << 4) | ((self.0 & (1<<7))<<11) | ((self.0 & (1<<31))<<12)},
            InstructionFormat::U => {self.0 & 0xFFFFF000},
            InstructionFormat::J => {(self.0.get_bits(21..=30)<<1) | (self.0.get_bits(20..=20)<<11) | (self.0 & 0x7F000) | (self.0.get_bits(31..=31)<<20)},
        }
    }
    fn _opcode_name(self) -> &'static str {
        find_instruction_desc(self).0
        // println!("Unknown instruction: {:x}", self.0);
        // "unknown" // Could return option ?
    }

    pub fn destination(self) -> Destination {
        match self.format() {
            InstructionFormat::R => Destination::CpuRegister(self.rd()),
            InstructionFormat::I => Destination::CpuRegister(self.rd()),
            InstructionFormat::S => Destination::CpuRegister(Register(0)),
            InstructionFormat::B => Destination::CpuRegister(Register(0)),
            InstructionFormat::U => Destination::CpuRegister(self.rd()),
            InstructionFormat::J => Destination::CpuRegister(self.rd()),
        }
    }
    // Returns the first input and tells if there is a second input (see `self.s2`)
    pub fn s1(self) -> (Destination, bool) {
        match self.format() {
            InstructionFormat::R => (Destination::CpuRegister(self.rs1()), true),
            InstructionFormat::I => (Destination::CpuRegister(self.rs1()), true),
            InstructionFormat::S => (Destination::Immediate(self.0), true), // S has 3 inputs and no outputs, so we put everything in one number
            InstructionFormat::B => (Destination::Immediate(self.0), false), // B has 3 inputs and no outputs, so we put everything in one number
            InstructionFormat::U => (Destination::Immediate(self.0 & 0xFFFFF000), false),
            InstructionFormat::J => (Destination::Immediate((self.0.get_bits(21..=30)<<1) | (self.0.get_bits(20..=20)<<11) | (self.0.get_bits(12..=19)<<12) | (self.0.get_bits(31..=31)<<20)), false),
        }
    }
    pub fn s2(self) -> Destination {
        match self.format() {
            InstructionFormat::R => Destination::CpuRegister(self.rs2()),
            InstructionFormat::I => Destination::Immediate(self.0.get_bits(20..=31)),
            InstructionFormat::S => {Destination::CpuRegister(self.rs2())},
            InstructionFormat::B => {println!("WARN: Trying to get s2 of a B format");Destination::Immediate(0)},
            InstructionFormat::U => {println!("WARN: Trying to get s2 of a U format");Destination::Immediate(0)}, // No rs2
            InstructionFormat::J => {println!("WARN: Trying to get s2 of a J format");Destination::Immediate(0)}, // No rs2
        }
    }
}
impl std::fmt::Debug for Instruction {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { 
        fmt.write_str(&format!("{:b} {:b} {:b} {:b}", self.opcode(), self.fun3(), self.fun7(), self.0))
    }
}
impl std::fmt::Display for Instruction {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let (s1, has_s2) = self.s1();
        let s2 = if has_s2 {self.s2()} else {Destination::Immediate(0)};
        fmt.write_str(&format!("{} {} {} {}", self._opcode_name(), self.destination(), s1, s2))
    }
}


#[derive(Debug)]
pub enum Destination {
    CpuRegister(Register),
    Immediate(u32),
}

impl std::fmt::Display for Destination {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = match self {
            Destination::CpuRegister(reg) => super::reg::REGS[reg.0 as usize].to_string(),
            Destination::Immediate(imm) => format!("{}", imm),
        };
        f.write_fmt(format_args!("{}", val))
    }
}




#[derive(Clone, Copy, Debug)]
pub struct InstructionMask(pub u32);
// impl InstructionMask {
//     // Returns true if the mask corresponds to the same instruction
//     // Like if opcode is same, fun3 and fun7 if there is one
//     pub fn is_mask(self, instruction: Instruction) -> bool {
//         let s = Instruction(self.0);
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



#[derive(Debug, Clone, Copy)]
pub enum InstructionFormat {
    R,I,S,B,U,J,
}



// rDestination, rS1, rS2
pub fn parse_r(i: Instruction) -> (Register, Register, Register) {
    (i.rd(),i.rs1(),i.rs2())
}

pub fn empty_fun(rs1:RegisterValue, rs2:RegisterValue) -> RegisterValue {
    dbg!(rs1,rs2);
    dbg!("Unsupported function !");
    0
}

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