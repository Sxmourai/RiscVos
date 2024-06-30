use std::{any::type_name, cell::OnceCell, collections::LinkedList};

use bit_field::BitField;

use crate::instructions;

struct VM {
    program: Vec<u8>,
    
}
impl VM {
    pub fn new(program: Vec<u8>) -> Self {
        Self {
            program,
        }
    }
    // More efficient than `(program[5] as u16) << 8|program[4] as u16` ?
    pub fn get_word(&self, idx: usize) -> u16 {
        unsafe {*((self.program.as_ptr() as *const u16).offset(idx as isize))}
    }
    pub fn get_dword(&self, idx: usize) -> u32 {
        unsafe {*((self.program.as_ptr() as *const u32).offset(idx as isize))}
    }
    pub fn get_T<T: Copy>(&self, idx: usize) -> T {
        unsafe {*((self.program.as_ptr() as *const T).offset(idx as isize))}
    }
}

bitfield::bitfield! {
    pub struct RInstruction(u32);
    impl Debug;
    pub opcode, _: 6, 0;
    pub rd, _: 11, 7;
    pub fun3, _: 14, 12;
    pub rs1, _: 19, 15;
    pub rs2, _: 24, 20;
    pub func7, _: 31, 25;
}
bitfield::bitfield! {
    pub struct IInstruction(u32);
    impl Debug;
    pub opcode, _: 6, 0;
    pub rd, _: 11, 7;
    pub fun3, _: 14, 12;
    pub rs1, _: 19, 15;
    pub imm, _: 31, 20;
}
bitfield::bitfield! {
    pub struct SInstruction(u32);
    impl Debug;
    pub opcode, _: 6, 0;
    pub lo_imm, _: 11, 7;
    pub fun3, _: 14, 12;
    pub rs1, _: 19, 15;
    pub rs2, _: 24, 20;
    pub hi_imm, _: 31, 25;
}
bitfield::bitfield! {
    pub struct BInstruction(u32);
    impl Debug;
    pub opcode, _: 6, 0;
    pub lo_imm, _: 11, 7;
    pub fun3, _: 14, 12;
    pub rs1, _: 19, 15;
    pub rs2, _: 24, 20;
    pub hi_imm, _: 31, 25;
}
#[derive(Debug, Clone, Copy)]
pub enum InstructionFormat {
    R,I,S,B,U,J,
}

const fn _mask(opcode: u32, fun3: u32, fun7: u32) -> InstructionMask {
    InstructionMask(opcode | fun3 << 12 | fun7 << 25)
}

// Big thanks to https://www.eg.bucknell.edu/~csci206/riscv-converter/Annotated_RISCV_Card.pdf
const INSTRUCTIONS_MASKS: [(&'static str, InstructionFormat, InstructionMask, fn(Instruction)); 63] = [
    ("lb",      InstructionFormat::I, _mask(0b0000011, 0b000, 0b0), empty_fun), // ! Do we need to set fun7 ?
    ("lh",      InstructionFormat::I, _mask(0b0000011, 0b001, 0b0), empty_fun),
    ("lw",      InstructionFormat::I, _mask(0b0000011, 0b010, 0b0), empty_fun),
    ("ld",      InstructionFormat::I, _mask(0b0000011, 0b011, 0b0), empty_fun),
    ("lbu",     InstructionFormat::I, _mask(0b0000011, 0b100, 0b0), empty_fun),
    ("lhu",     InstructionFormat::I, _mask(0b0000011, 0b110, 0b0), empty_fun),
    ("lwu",     InstructionFormat::I, _mask(0b0000011, 0b111, 0b0), empty_fun),
    
    ("fence",   InstructionFormat::I, _mask(0b0001111, 0b000, 0b0), empty_fun),
    ("fence.i", InstructionFormat::I, _mask(0b0001111, 0b001, 0b0), empty_fun),
    
    ("addi",    InstructionFormat::I, _mask(0b0010011, 0b000, 0b0), empty_fun),
    ("slli",    InstructionFormat::I, _mask(0b0010011, 0b001, 0b0), empty_fun), // Has funct7 ??
    ("slti",    InstructionFormat::I, _mask(0b0010011, 0b010, 0b0), empty_fun),
    ("sltiu",   InstructionFormat::I, _mask(0b0010011, 0b011, 0b0), empty_fun),
    ("xori",    InstructionFormat::I, _mask(0b0010011, 0b100, 0b0), empty_fun),
    ("srli",    InstructionFormat::I, _mask(0b0010011, 0b101, 0b0000000), empty_fun),
    ("srai",    InstructionFormat::I, _mask(0b0010011, 0b101, 0b0100000), empty_fun),
    ("ori",     InstructionFormat::I, _mask(0b0010011, 0b110, 0b0), empty_fun),
    ("andi",    InstructionFormat::I, _mask(0b0010011, 0b111, 0b0), empty_fun),
    
    ("auipc",   InstructionFormat::U, _mask(0b0010111, 0b000, 0b0), empty_fun),
    
    ("addiw",   InstructionFormat::I, _mask(0b0011011, 0b000, 0b0), empty_fun),
    ("slliw",   InstructionFormat::I, _mask(0b0011011, 0b001, 0b0000000), empty_fun),
    ("srliw",   InstructionFormat::I, _mask(0b0011011, 0b101, 0b0000000), empty_fun),
    ("sraiw",   InstructionFormat::I, _mask(0b0011011, 0b101, 0b0100000), empty_fun),

    ("sb",   InstructionFormat::S, _mask(0b0100011, 0b000, 0b0), empty_fun),
    ("sh",   InstructionFormat::S, _mask(0b0100011, 0b001, 0b0), empty_fun),
    ("sw",   InstructionFormat::S, _mask(0b0100011, 0b010, 0b0), empty_fun),
    ("sd",   InstructionFormat::S, _mask(0b0100011, 0b011, 0b0), empty_fun),

    ("sb",   InstructionFormat::R, _mask(0b0100011, 0b000, 0b0), empty_fun),
    ("sh",   InstructionFormat::R, _mask(0b0100011, 0b001, 0b0), empty_fun),
    ("sw",   InstructionFormat::R, _mask(0b0100011, 0b010, 0b0), empty_fun),
    ("sd",   InstructionFormat::R, _mask(0b0100011, 0b011, 0b0), empty_fun),
    ("add",  InstructionFormat::R, _mask(0b0110011, 0b000, 0b0000000), add),
    ("sub",  InstructionFormat::R, _mask(0b0110011, 0b000, 0b0100000), empty_fun),
    ("sil",  InstructionFormat::R, _mask(0b0110011, 0b001, 0b0000000), empty_fun),
    ("slt",  InstructionFormat::R, _mask(0b0110011, 0b010, 0b0000000), empty_fun),
    ("sltu", InstructionFormat::R, _mask(0b0110011, 0b011, 0b0000000), empty_fun),
    ("xor",  InstructionFormat::R, _mask(0b0110011, 0b100, 0b0000000), empty_fun),
    ("srl",  InstructionFormat::R, _mask(0b0110011, 0b101, 0b0000000), empty_fun),
    ("sra",  InstructionFormat::R, _mask(0b0110011, 0b101, 0b0100000), empty_fun),
    ("or",   InstructionFormat::R, _mask(0b0110011, 0b110, 0b0000000), empty_fun),
    ("and",  InstructionFormat::R, _mask(0b0110011, 0b111, 0b0000000), empty_fun),

    ("lui",  InstructionFormat::U, _mask(0b0110111, 0b0, 0b0), empty_fun),
    
    ("addw", InstructionFormat::R, _mask(0b0111011, 0b000, 0b0000000), empty_fun),
    ("subw", InstructionFormat::R, _mask(0b0111011, 0b000, 0b0100000), empty_fun),
    ("sllw", InstructionFormat::R, _mask(0b0111011, 0b001, 0b0000000), empty_fun),
    ("srllw",InstructionFormat::R, _mask(0b0111011, 0b101, 0b0000000), empty_fun),
    ("sraw", InstructionFormat::R, _mask(0b0111011, 0b101, 0b0100000), empty_fun),

    ("beq",  InstructionFormat::B, _mask(0b1100011, 0b000, 0b0), empty_fun),
    ("bne",  InstructionFormat::B, _mask(0b1100011, 0b001, 0b0), empty_fun),
    ("bit",  InstructionFormat::B, _mask(0b1100011, 0b100, 0b0), empty_fun),
    ("bge",  InstructionFormat::B, _mask(0b1100011, 0b101, 0b0), empty_fun),
    ("bitu", InstructionFormat::B, _mask(0b1100011, 0b110, 0b0), empty_fun),
    ("bgeu", InstructionFormat::B, _mask(0b1100011, 0b111, 0b0), empty_fun),

    ("jalr", InstructionFormat::I, _mask(0b1100111, 0b111, 0b0), empty_fun),
    
    ("jal",  InstructionFormat::J, _mask(0b1101111, 0b0, 0b0), empty_fun),

    ("ecall",  InstructionFormat::I, _mask(0b1110011, 0b0, 0b0), empty_fun), // Immediates (fun7)
    ("ebreak", InstructionFormat::I, _mask(0b1110011, 0b0, 0b1), empty_fun),

    ("CSRRW",  InstructionFormat::I, _mask(0b1110011, 0b001, 0b0), empty_fun),
    ("CSRRS",  InstructionFormat::I, _mask(0b1110011, 0b010, 0b0), empty_fun),
    ("CSRRC",  InstructionFormat::I, _mask(0b1110011, 0b011, 0b0), empty_fun),
    ("CSRRWI", InstructionFormat::I, _mask(0b1110011, 0b101, 0b0), empty_fun),
    ("CSRRSI", InstructionFormat::I, _mask(0b1110011, 0b110, 0b0), empty_fun),
    ("CSRRCI", InstructionFormat::I, _mask(0b1110011, 0b111, 0b0), empty_fun),
];

// rDestination, rS1, rS2
fn parse_r(i: Instruction) -> (Register, Register, Register) {
    (i.rd(),i.rs1(),i.rs2())
}



fn add(ins: Instruction) {
    dbg!(ins);
}
fn sub(ins: Instruction) {

}
fn empty_fun(ins: Instruction) {
    dbg!(ins);
    println!("Unsupported function ! {}", ins);
}
type InstructionDescription = (&'static str, InstructionFormat, InstructionMask, fn(Instruction));
type _ReverseInstructionsMasks = [Vec<InstructionDescription>; 127];
static mut REVERSE_INSTRUCTIONS_MASKS: OnceCell<_ReverseInstructionsMasks> = OnceCell::new();
fn set_instructions_funcs() {
    let mut instru_funcs: _ReverseInstructionsMasks = std::array::from_fn(|_| Vec::new());
    for (i,(name, format, mask, fun)) in INSTRUCTIONS_MASKS.iter().enumerate() {
        let opcode = Instruction(mask.0).opcode();
        instru_funcs[opcode as usize].push((name, *format, *mask, *fun));
    }
    unsafe{REVERSE_INSTRUCTIONS_MASKS.set(instru_funcs).unwrap()}
}


fn exec_func(instruction: Instruction) {
    let instructions = &unsafe{REVERSE_INSTRUCTIONS_MASKS.get().unwrap()}[instruction.opcode() as usize];
    let (_name, _fmt, _mask, fun) = find_instruction_desc(instruction);
    fun(instruction);
    // println!("Instruction not found \"{:?}\"", instruction);
}



fn get_from_opcode(opcode:u32) -> &'static Vec<InstructionDescription> {
    unsafe{&REVERSE_INSTRUCTIONS_MASKS.get().unwrap()[opcode as usize]}
}
fn try_find_instruction_desc(inst: Instruction) -> Option<InstructionDescription> {
    let opcode = inst.opcode();
    let neighbors = unsafe{&REVERSE_INSTRUCTIONS_MASKS.get().unwrap()[opcode as usize]};
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
    println!("{:?}", inst);
    None
}
fn find_instruction_desc(inst: Instruction) -> InstructionDescription {
    try_find_instruction_desc(inst).unwrap()
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
    fn _raw_rd(self) -> u32 {
        self.0.get_bits(7..=11)
    }
    // 4 bits
    // Register source 1
    pub fn rs1(self) -> Register {
        Register::new(self._raw_rs1())
    }
    fn _raw_rs1(self) -> u32 {
        self.0.get_bits(15..=19)
    }
    // 4 bits
    // Register source 2
    pub fn rs2(self) -> Register {
        Register::new(self._raw_rs2())
    }
    fn _raw_rs2(self) -> u32 {
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
}
impl std::fmt::Debug for Instruction {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { 
        fmt.write_str(&format!("{:b} {:b} {:b} {:b}", self.opcode(), self.fun3(), self.fun7(), self.0))
    }
}
impl std::fmt::Display for Instruction {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { 
        fmt.write_str(&format!("{} {} {} {}", self._opcode_name(), self.rd(), self.rs1(), self.rs2()))
    }
}
#[derive(Clone, Copy, Debug)]
pub struct InstructionMask(pub u32);
impl InstructionMask {
    // Returns true if the mask corresponds to the same instruction
    // Like if opcode is same, fun3 and fun7 if there is one
    pub fn is_mask(self, instruction: Instruction) -> bool {
        let s = Instruction(self.0);
        if instruction.opcode() == s.opcode() {
            return match instruction.format() {
                InstructionFormat::R => {s.fun3() == instruction.fun3() && s.fun7() == instruction.fun7()},
                InstructionFormat::I => {s.fun3() == instruction.fun3()},
                InstructionFormat::S => {s.fun3() == instruction.fun3()},
                InstructionFormat::B => {s.fun3() == instruction.fun3()},
                InstructionFormat::U => {true},
                InstructionFormat::J => {true},
            }
        }
        false
    }
}

const REGS: [&'static str; 64] = ["zero", "ra", "sp", "gp", "tp", "t0", "t1", "t2", 
    "s0", "s1", // or "fp"
    "a0", "a1", "a2", "a3", "a4", "a5", "a6", "a7", 
    "s2", "s3", "s4", "s5", "s6", "s7", "s8", "s9", "s10", "s11",
    "t3","t4","t5","t6",
    "ft0", "ft1", "ft2", "ft3", "ft4", "ft5", "ft6", "ft7", 
    "fs0", "fs1",
    "fa0", "fa1",
    "fa2","fa3","fa4","fa5","fa6","fa7",
    "fs2", "fs3", "fs4", "fs5", "fs6", "fs7", "fs8", "fs9", "fs10", "fs11",
    "ft8","ft9","ft10","ft11",
];
#[derive(Clone, Copy, Debug)]
pub struct Register(pub u32);
impl Register {
    pub fn new(reg: u32) -> Self {
        if reg as usize > REGS.len() {todo!()}
        let s = Self(reg);
        s
    }
}
impl std::fmt::Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        f.write_str(REGS[self.0 as usize])
    }
}
pub fn run(program: Vec<u8>) {
    set_instructions_funcs();
    let vm = VM::new(program);
    for i in 0..vm.program.len()/core::mem::size_of::<u32>() {
        let instruction = Instruction::new(vm.get_dword(i));
        // if instruction.is_none() {break;} // The file currently has some unknown instructions
        let instruction = instruction.unwrap();
        println!("{}",instruction);
        exec_func(instruction)
    }
}