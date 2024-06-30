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
        unsafe {*((self.program.as_ptr() as *const u32).offset(idx as isize))}
    }
    pub fn get_T<T: Copy>(&self, idx: usize) -> T {
        unsafe {*((self.program.as_ptr() as *const T).offset(idx as isize))}
    }
}


pub fn run(program: Vec<u8>) {
    crate::cpu::instructions::set_instructions_funcs();
    let vm = VM::new(program);
    for i in 0..vm.program.len()/core::mem::size_of::<u32>() {
        let instruction = Instruction::new(vm.get_dword(i));
        // if instruction.is_none() {break;} // The file currently has some unknown instructions
        let instruction = instruction.unwrap();
        println!("{}",instruction);
        crate::cpu::instructions::exec_func(instruction)
    }
}