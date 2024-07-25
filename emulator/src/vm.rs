use crate::cpu::instructions::Instruction;

pub struct VM {
    pub memory: Vec<u8>,
    pub cpu: crate::cpu::CPU,
}
impl VM {
    pub fn new(memory: Vec<u8>) -> Self {
        crate::cpu::instructions::set_instructions_funcs();
        Self {
            memory,
            cpu: crate::cpu::CPU::new(),
        }
    }
    // More efficient than `(program[5] as u16) << 8|program[4] as u16` ?
    pub fn get_dword(&self, idx: usize) -> u32 {
        self.get_any::<u32>(idx)
    }
    pub fn set_dword(&mut self, idx: usize, value: u32) {
        self.set_any::<u32>(idx, value)
    }
    pub fn get_any<T: Copy>(&self, idx: usize) -> T {
        assert!(idx < self.memory.len()/core::mem::size_of::<T>());
        unsafe {*((self.memory.as_ptr() as *const T).offset(idx as isize))}
    }
    pub fn set_any<T: Copy>(&mut self, idx: usize, value: T) {
        assert!(idx < self.memory.len()/core::mem::size_of::<T>());
        unsafe {*((self.memory.as_mut_ptr() as *mut T).offset(idx as isize)) = value}
    }
    pub fn run(&mut self) {
        loop {
            if self.cpu.pc as usize >= self.memory.len() {break;}
            // Fetch
            let inst = Instruction::new(self.get_dword(self.cpu.pc as usize/core::mem::size_of::<Instruction>()));
            if inst.is_none() {println!("The program didn't enter in a end-loop ! This would've led to UB");break;}
            let inst = inst.unwrap(); // Unwrap unchecked
            // Execute
            println!("{}\t - ",inst);
            let (_name, _fmt, _mask, fun) = crate::cpu::instructions::find_instruction_desc(inst);
            
            let (s1, has_s2) = match inst.s1() {
                (crate::cpu::instructions::Destination::CpuRegister(reg), has_s2) => (self.cpu[reg], has_s2),
                (crate::cpu::instructions::Destination::Immediate(imm), has_s2) => (imm, has_s2),
            };
            let s2 = if has_s2 {match inst.s2() {
                crate::cpu::instructions::Destination::CpuRegister(reg) => self.cpu[reg],
                crate::cpu::instructions::Destination::Immediate(imm) => imm,
            }} else {0};
            let d = fun(self, s1, s2);
            
            // println!("{}={}\t{}={}\t-> {}={}", inst.s1().0,s1,inst.s2(),s2,inst.destination(),d);
            match inst.destination() {
                crate::cpu::instructions::Destination::CpuRegister(reg) => self.cpu[reg] = d,
                crate::cpu::instructions::Destination::Immediate(imm) => self.set_dword(imm as usize/4, d),
            };
            self.cpu.pc += core::mem::size_of::<Instruction>() as u32;
            self.cpu.zero = 0; // Currently we need to set it manually
        }
        dbg!(&self.cpu);
    }
    pub fn disasm(&self) {
        for i in 0..self.memory.len()/4 {
            // Fetch
            let inst = Instruction::new(self.get_dword(i as usize));
            if inst.is_none() {println!("The program didn't enter in a end-loop ! This would've led to UB");break;}
            let inst = inst.unwrap(); // Unwrap unchecked
            // Execute
            println!("{}\t - ",inst);
            let (_name, _fmt, _mask, fun) = crate::cpu::instructions::find_instruction_desc(inst);
            
            let (s1, has_s2) = match inst.s1() {
                (crate::cpu::instructions::Destination::CpuRegister(reg), has_s2) => (self.cpu[reg], has_s2),
                (crate::cpu::instructions::Destination::Immediate(imm), has_s2) => (imm, has_s2),
            };
            let s2 = if has_s2 {match inst.s2() {
                crate::cpu::instructions::Destination::CpuRegister(reg) => self.cpu[reg],
                crate::cpu::instructions::Destination::Immediate(imm) => imm,
            }} else {0};
            
            // println!("{}={}\t{}={}\t-> {}={}", inst.s1().0,s1,inst.s2(),s2,inst.destination(),d);
        }
    }
}

pub fn run(program: Vec<u8>) {
    VM::new(program).run();
}