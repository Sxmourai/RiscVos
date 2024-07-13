use std::{cell::OnceCell, collections::LinkedList};

use bit_field::BitField;

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
    pub fn get_any<T: Copy>(&self, idx: usize) -> T {
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
pub enum InstructionFormat {
    R,I,S,B,U,J,
}

const fn _mask(opcode: u32, fun3: u32, fun7: u32) -> u32 {
    opcode | fun3 << 12 | fun7 << 25
}

// Big thanks to https://www.eg.bucknell.edu/~csci206/riscv-converter/Annotated_RISCV_Card.pdf
const INSTRUCTIONS_MASKS: [(&'static str, InstructionFormat, u32); 63] = [
    ("lb",      InstructionFormat::I, _mask(0b0000011, 0b000, 0b0)), // ! Do we need to set fun7 ?
    ("lh",      InstructionFormat::I, _mask(0b0000011, 0b001, 0b0)),
    ("lw",      InstructionFormat::I, _mask(0b0000011, 0b010, 0b0)),
    ("ld",      InstructionFormat::I, _mask(0b0000011, 0b011, 0b0)),
    ("lbu",     InstructionFormat::I, _mask(0b0000011, 0b100, 0b0)),
    ("lhu",     InstructionFormat::I, _mask(0b0000011, 0b110, 0b0)),
    ("lwu",     InstructionFormat::I, _mask(0b0000011, 0b111, 0b0)),
    
    ("fence",   InstructionFormat::I, _mask(0b0001111, 0b000, 0b0)),
    ("fence.i", InstructionFormat::I, _mask(0b0001111, 0b001, 0b0)),
    
    ("addi",    InstructionFormat::I, _mask(0b0010011, 0b000, 0b0)),
    ("slli",    InstructionFormat::I, _mask(0b0010011, 0b001, 0b0)), // Has funct7 ??
    ("slti",    InstructionFormat::I, _mask(0b0010011, 0b010, 0b0)),
    ("sltiu",   InstructionFormat::I, _mask(0b0010011, 0b011, 0b0)),
    ("xori",    InstructionFormat::I, _mask(0b0010011, 0b100, 0b0)),
    ("srli",    InstructionFormat::I, _mask(0b0010011, 0b101, 0b0000000)),
    ("srai",    InstructionFormat::I, _mask(0b0010011, 0b101, 0b0100000)),
    ("ori",     InstructionFormat::I, _mask(0b0010011, 0b110, 0b0)),
    ("andi",    InstructionFormat::I, _mask(0b0010011, 0b111, 0b0)),
    
    ("auipc",   InstructionFormat::U, _mask(0b0010111, 0b000, 0b0)),
    
    ("addiw",   InstructionFormat::I, _mask(0b0011011, 0b000, 0b0)),
    ("slliw",   InstructionFormat::I, _mask(0b0011011, 0b001, 0b0000000)),
    ("srliw",   InstructionFormat::I, _mask(0b0011011, 0b101, 0b0000000)),
    ("sraiw",   InstructionFormat::I, _mask(0b0011011, 0b101, 0b0100000)),

    ("sb",   InstructionFormat::S, _mask(0b0100011, 0b000, 0b0)),
    ("sh",   InstructionFormat::S, _mask(0b0100011, 0b001, 0b0)),
    ("sw",   InstructionFormat::S, _mask(0b0100011, 0b010, 0b0)),
    ("sd",   InstructionFormat::S, _mask(0b0100011, 0b011, 0b0)),

    ("sb",   InstructionFormat::R, _mask(0b0100011, 0b000, 0b0)),
    ("sh",   InstructionFormat::R, _mask(0b0100011, 0b001, 0b0)),
    ("sw",   InstructionFormat::R, _mask(0b0100011, 0b010, 0b0)),
    ("sd",   InstructionFormat::R, _mask(0b0100011, 0b011, 0b0)),
    ("add",  InstructionFormat::R, _mask(0b0110011, 0b000, 0b0000000)),
    ("sub",  InstructionFormat::R, _mask(0b0110011, 0b000, 0b0100000)),
    ("sil",  InstructionFormat::R, _mask(0b0110011, 0b001, 0b0000000)),
    ("slt",  InstructionFormat::R, _mask(0b0110011, 0b010, 0b0000000)),
    ("sltu", InstructionFormat::R, _mask(0b0110011, 0b011, 0b0000000)),
    ("xor",  InstructionFormat::R, _mask(0b0110011, 0b100, 0b0000000)),
    ("srl",  InstructionFormat::R, _mask(0b0110011, 0b101, 0b0000000)),
    ("sra",  InstructionFormat::R, _mask(0b0110011, 0b101, 0b0100000)),
    ("or",   InstructionFormat::R, _mask(0b0110011, 0b110, 0b0000000)),
    ("and",  InstructionFormat::R, _mask(0b0110011, 0b111, 0b0000000)),

    ("lui",  InstructionFormat::U, _mask(0b0110111, 0b0, 0b0)),
    
    ("addw", InstructionFormat::R, _mask(0b0111011, 0b000, 0b0000000)),
    ("subw", InstructionFormat::R, _mask(0b0111011, 0b000, 0b0100000)),
    ("sllw", InstructionFormat::R, _mask(0b0111011, 0b001, 0b0000000)),
    ("srllw",InstructionFormat::R, _mask(0b0111011, 0b101, 0b0000000)),
    ("sraw", InstructionFormat::R, _mask(0b0111011, 0b101, 0b0100000)),

    ("beq",  InstructionFormat::B, _mask(0b1100011, 0b000, 0b0)),
    ("bne",  InstructionFormat::B, _mask(0b1100011, 0b001, 0b0)),
    ("bit",  InstructionFormat::B, _mask(0b1100011, 0b100, 0b0)),
    ("bge",  InstructionFormat::B, _mask(0b1100011, 0b101, 0b0)),
    ("bitu", InstructionFormat::B, _mask(0b1100011, 0b110, 0b0)),
    ("bgeu", InstructionFormat::B, _mask(0b1100011, 0b111, 0b0)),

    ("jalr", InstructionFormat::I, _mask(0b1100111, 0b111, 0b0)),
    
    ("jal",  InstructionFormat::J, _mask(0b1101111, 0b0, 0b0)),

    ("ecall",  InstructionFormat::I, _mask(0b1110011, 0b0, 0b0)), // Immediates (fun7)
    ("ebreak", InstructionFormat::I, _mask(0b1110011, 0b0, 0b1)),

    ("CSRRW",  InstructionFormat::I, _mask(0b1110011, 0b001, 0b0)),
    ("CSRRS",  InstructionFormat::I, _mask(0b1110011, 0b010, 0b0)),
    ("CSRRC",  InstructionFormat::I, _mask(0b1110011, 0b011, 0b0)),
    ("CSRRWI", InstructionFormat::I, _mask(0b1110011, 0b101, 0b0)),
    ("CSRRSI", InstructionFormat::I, _mask(0b1110011, 0b110, 0b0)),
    ("CSRRCI", InstructionFormat::I, _mask(0b1110011, 0b111, 0b0)),
];

fn add(ins: Instruction) {

}
fn sub(ins: Instruction) {

}
fn empty_fun(ins: Instruction) {
    todo!()
}

fn exec_func(instruction: Instruction) {
    let funcs: [LinkedList<(Instruction, fn(Instruction))>; 63] = std::array::from_fn(|_| LinkedList::new());

}


// Elements are linked lists of a fun3 and fun7 and the str name
fn get_ops() -> [LinkedList<(Instruction, &'static str)>; 128] {
    const _L: LinkedList<(Instruction, &'static str)> = LinkedList::new();
    let mut ops = [_L; 128];
    let mut i = 0;
    while i < INSTRUCTIONS_MASKS.len() { // Can't use for loop :c
        let (name, format, op) = &INSTRUCTIONS_MASKS[i];
        let inst = Instruction(*op);
        let list = &mut ops[inst.opcode() as usize];
        list.push_back((inst, name));
        i+=1;
    }
    ops
}
static mut OPS: OnceCell<[LinkedList<(Instruction, &'static str)>; 128]> = OnceCell::new();

fn get_from_opcode(opcode:u32) -> &'static LinkedList<(Instruction, &'static str)> {
    unsafe{&OPS.get().unwrap()[opcode as usize]}
}

#[derive(Clone, Copy)]
pub struct Instruction(pub u32);
impl Instruction {
    // 7 bits
    pub fn opcode(self) -> u32 {
        self.0.get_bits(0..=6)
    }
    // Only usefull if instruction is of type R, I, U, J
    // Destination register
    // 4 bits
    pub fn rd(self) -> u32 {
        self.0.get_bits(7..=11)
    }
    // 4 bits
    // Register source 1
    pub fn rs1(self) -> u32 {
        self.0.get_bits(15..=19)
    }
    // 4 bits
    // Register source 2
    pub fn rs2(self) -> u32 {
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
        match self.opcode() {
            _ => todo!(),
        }
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
        for (mask, name) in get_from_opcode(self.opcode()) {
            if mask.fun3() == self.fun3() && mask.fun7() == self.fun7() {
                return name
            }
        }
        println!("Unknown instruction: {:x}", self.0);
        "unknown" // Could return option ?
    }
    fn _reg_to_str(reg: u32) -> &'static str {
        ["zero", "ra", "sp", "gp", "tp", "t0", "t1", "t2", 
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
        ][reg as usize]
        // match reg {
        //     0 => "zero",
        //     1 => "ra"
        //     _ => {todo!()}
        // }
    }
}
impl std::fmt::Debug for Instruction {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { 
        fmt.write_str(&format!("{} {} {} {}", self._opcode_name(), Self::_reg_to_str(self.rd()), Self::_reg_to_str(self.rs1()), Self::_reg_to_str(self.rs2())))
    }
}

pub fn disasm(program: Vec<u8>) -> Option<String> {
    unsafe{OPS.set(get_ops()).unwrap()};
    let vm = VM::new(program);
    let mut parsed = String::new();
    use std::fmt::Write;
    for i in 0..vm.program.len()/2 {
        let instruction = Instruction(vm.get_dword(i));
        if instruction._opcode_name() == "unknown" {break;} // The file currently has some unknown instructions
        writeln!(parsed, "{:?}",instruction).ok()?;
    }
    Some(parsed)
}


pub fn run(program: Vec<u8>) {
    unsafe{OPS.set(get_ops()).unwrap()};
    let vm = VM::new(program);
    for i in 0..vm.program.len()/2 {
        let instruction = Instruction(vm.get_dword(i));
        if instruction._opcode_name() == "unknown" {break;} // The file currently has some unknown instructions
        println!("{:?}",instruction);
    }
}