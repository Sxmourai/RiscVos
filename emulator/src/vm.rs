use crate::cpu::instructions::Instruction;

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
        // assert!(idx < self.program.len()/4);
        unsafe {*((self.program.as_ptr() as *const u32).offset(idx as isize))}
    }
    pub fn set_dword(&mut self, idx: usize, value: u32) {
        unsafe {*((self.program.as_mut_ptr() as *mut u32).offset(idx as isize)) = value}
    }
    pub fn get_T<T: Copy>(&self, idx: usize) -> T {
        unsafe {*((self.program.as_ptr() as *const T).offset(idx as isize))}
    }
}


pub fn run(program: Vec<u8>) {
    crate::cpu::instructions::set_instructions_funcs();
    let mut vm = VM::new(program);
    let mut cpu = crate::cpu::CPU::new();
    loop {
        if cpu.tp as usize >= vm.program.len() {break;}
        // Fetch
        let inst = Instruction::new(vm.get_dword(cpu.tp as usize/core::mem::size_of::<Instruction>()));
        if inst.is_none() {println!("The program didn't enter in a end-loop ! This would've led to UB");break;}
        let inst = inst.unwrap(); // Unwrap unchecked
        // Execute
        println!("{}",inst);
        
        let (_name, _fmt, _mask, fun) = crate::cpu::instructions::find_instruction_desc(inst);
        
        let s1 = match inst.s1() {
            crate::cpu::instructions::Destination::CpuRegister(reg) => cpu[reg],
            crate::cpu::instructions::Destination::Immediate(imm) => imm,
        };
        let s2 = match inst.s2() {
            crate::cpu::instructions::Destination::CpuRegister(reg) => cpu[reg],
            crate::cpu::instructions::Destination::Immediate(imm) => imm,
        };
        let d = fun(s1, s2);
        println!("{:?} {:b}", (s1,s2,inst.destination(), d,inst.s1(),inst.s2()), inst.0);
        match inst.destination() {
            crate::cpu::instructions::Destination::CpuRegister(reg) => cpu[reg] = d,
            crate::cpu::instructions::Destination::Immediate(imm) => vm.set_dword(imm as usize/4, d),
        };
        cpu.tp += core::mem::size_of::<Instruction>() as u32;
    }
    dbg!(cpu);
}