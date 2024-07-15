// For cpu exceptions and interrupts

use crate::*;

pub enum InterruptsMode {
    Direct,
    Vectored,
}
#[repr(C)]
#[derive(Clone)]
pub struct TrapFrame {
	pub regs:   [u64; 32],
    // Float registers
	pub fregs:  [u64; 32],
	pub satp:   u64,
	pub trap_stack: *mut u8,
	pub hartid: u64,
}

extern "C" fn _trap_vector() {
    
}


// unsafe fn csrr(read_reg: &'static str) -> u64 {
// }

pub fn save_context(addr: usize) {
    // Mscratch is typically used to hold a pointer to a machine-mode hart-local context space and swapped with a
    // user register upon entry to an M-mode trap handler.
    // unsafe{core::arch::asm!("csrrw mscratch, t6");} // Atomically swap mscratch and t6
    // Save all registers
    // for i in 0..32 {
    //     core::arch::asm!(concat!("ld x{}, 0x64",i));
    // }
    macro_rules! save_registers {
        ($base_addr: ident, $($reg:expr),*) => {
            $(
                core::arch::asm!(
                    concat!("sd x", stringify!($reg), ", ", stringify!($reg), "*8({0})"),
                    in(reg) $base_addr,
                );
            )*
        }
    }
    unsafe {
        save_registers!(addr, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31);
    }
    dbg!(addr);
}
#[inline(never)]
pub fn load_context(addr: usize) {
    macro_rules! load_registers {
        ($base_addr: ident, $($reg:expr),*) => {
            $(
                core::arch::asm!(
                    concat!("ld x", stringify!($reg), ", ", stringify!($reg), "*8({0})"),
                    in(reg) $base_addr,
                );
            )*
        }
    }
    unsafe {
        load_registers!(addr, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31);
    }
}
pub enum Interrupts {
    SupervisorSoftwareInterrupt = 1,
    MachineSoftwareInterrupt = 3,
    SupervisorTimerInterrupt = 5,
    MachineTimerInterrupt = 7,
    SupervisorExternalInterrupt = 9,
    MachineExternalInterrupt = 11,
    CounterOverflowInterrupt = 13,
}
pub enum Exceptions {
    InstructionAddressMisaligned = 0,
    InstructionAccessFault = 1,
    IllegalInstruction = 2,
    Breakpoint = 3,
    LoadAddressMisaligned = 4,
    LoadAccessFault = 5,
    StoreOrAMOAddressMisaligned = 6,
    StoreOrAMOAccessFault = 7,
    EnvironmentCallFromUmode = 8,
    EnvironmentCallFromSmode = 9,
    EnvironmentCallFromMmode = 11,
    InstructionPageFault = 12,
    LoadPageFault = 13,
    StoreOrAMOPageFault = 15,
    SoftwareCheck = 18,
    HardwareError = 19,
}


#[no_mangle]
unsafe extern "C" fn trap_vector(epc: usize,
    tval: usize,
    cause: usize,
    hart: usize,
    status: usize,
    frame: &mut TrapFrame) -> usize {
    let cause = csrr!("mcause", u64);
    let is_interrupt = cause & 1u64<<63 != 0;
    let id = cause & 0xFF;
    crate::print!("Trap: {}\t", id);
    let mtval = csrr!("mtval", u64);
    dbg!(mtval);
    if is_interrupt {
        match id {
            1 => {println!("Supervisor software interrupt")},
            3 => {println!("Machine software interrupt")},
            5 => {println!("Supervisor timer interrupt")},
            7 => {println!("Machine timer interrupt")},
            9 => {println!("Supervisor external interrupt")},
            11 => {println!("Machine external interrupt")},
            13 => {println!("Counter overflow interrupt")},
            ..16 => {println!("Custom interrupt !")},
            _ => {println!("Interrupt cause: {:b}", id);},
        }
    } else {
        match id {
            0 => {println!("Instruction address misaligned: {}", mtval)},
            1 => {println!("Instruction access fault: {}", mtval)},
            2 => {println!("Illegal instruction: {}", mtval)},
            3 => {println!("Breakpoint: {}", mtval)},
            4 => {println!("Load address misaligned: {}", mtval)},
            5 => {println!("Load access fault: {}", mtval)},
            6 => {println!("Store/AMO address misaligned: {}", mtval)},
            7 => {println!("Store/AMO access fault: {}", mtval)},
            8 => {println!("Environment call from U-mode: {}", mtval)},
            9 => {println!("Environment call from S-mode: {}", mtval)},
            11 => {println!("Environment call from M-mode: {}", mtval)},
            12 => {println!("Instruction page fault: {}", mtval)},
            13 => {println!("Load page fault: {}", mtval)},
            15 => {println!("Store/AMO page fault: {}", mtval)},
            18 => {println!("Software check: {}", mtval)},
            19 => {println!("Hardware error: {}", mtval)},
            _ => {dbg!(cause, mtval);},
        }
    }
    for i in 0..1472762 {}
    epc
}

extern "C" {
    fn asm_trap_vector() -> ();
}
#[no_mangle]
pub extern "C" fn abort() {
    dbg!(csrr!("mcause"), csrr!("mtval"), csrr!("mscratch"));
    dbg!(csrr!("mepc"), csrr!("mtvec"));
    println!("Aborting...");
    // loop {}
} 

core::arch::global_asm!("
.option norvc
.altmacro
.set NUM_GP_REGS, 32  # Number of registers per context
.set NUM_FP_REGS, 32
.set REG_SIZE, 8   # Register size (in bytes)
.set MAX_CPUS, 8   # Maximum number of CPUs

# Use macros for saving and restoring multiple registers
.macro save_gp i, basereg=t6
	sd	x\\i, ((\\i)*REG_SIZE)(\\basereg)
.endm
.macro load_gp i, basereg=t6
	ld	x\\i, ((\\i)*REG_SIZE)(\\basereg)
.endm
.macro save_fp i, basereg=t6
	fsd	f\\i, ((NUM_GP_REGS+(\\i))*REG_SIZE)(\\basereg)
.endm
.macro load_fp i, basereg=t6
	fld	f\\i, ((NUM_GP_REGS+(\\i))*REG_SIZE)(\\basereg)
.endm

.global asm_trap_vector
asm_trap_vector:
# All registers are volatile here, we need to save them
# before we do anything.
csrrw	t6, mscratch, t6
# csrrw will atomically swap t6 into mscratch and the old
# value of mscratch into t6. This is nice because we just
# switched values and didn't destroy anything -- all atomically!
# in cpu.rs we have a structure of:
#  32 gp regs		0
#  32 fp regs		256
#  SATP register	512
#  Trap stack       520
#  CPU HARTID		528
# We use t6 as the temporary register because it is the very
# bottom register (x31)
.set 	i, 1
.rept	30
	save_gp	%i
	.set	i, i+1
.endr

# Save the actual t6 register, which we swapped into
# mscratch
mv		t5, t6
csrr	t6, mscratch
save_gp 31, t5

# Restore the kernel trap frame into mscratch
csrw	mscratch, t5
#!-----------------------------------------------
call abort

# Get ready to go into Rust (trap.rs)
# We don't want to write into the user's stack or whomever
# messed with us here.
csrr	a0, mepc
csrr	a1, mtval
csrr	a2, mcause
csrr	a3, mhartid
csrr	a4, mstatus
mv		a5, t5
ld		sp, 520(a5)
call	trap_vector

# When we get here, we've returned from m_trap, restore registers
# and return.
# m_trap will return the return address via a0.

csrw	mepc, a0

# Now load the trap frame back into t6
csrr	t6, mscratch

# Restore all GP registers
.set	i, 1
.rept	31
	load_gp %i
	.set	i, i+1
.endr

# Since we ran this loop 31 times starting with i = 1,
# the last one loaded t6 back to its original value.

mret");


pub fn init() {
    println!("Initialising traps...");
    let trap_vector_ptr = asm_trap_vector as u64 & !(0b11);
    let mut supervisor_mstatus = MSTATUS(0);
    supervisor_mstatus.set_mpp(PrivilegeLevel::Supervisor as u64);
    supervisor_mstatus.set_mpie(true); // I think mpie should be set anyway, because we can't have spie and not mpie (see ISA / doc)
    supervisor_mstatus.set_spie(true);
    supervisor_mstatus.set_mie(true);
    supervisor_mstatus.set_sie(true);
    let trap_frame_addr = kalloc(1).unwrap() as u64;
    let interrupts_enable = (1<<Interrupts::SupervisorTimerInterrupt as u64) | (1<<Interrupts::SupervisorSoftwareInterrupt as u64) | (1<<Interrupts::SupervisorExternalInterrupt as u64);
    unsafe{
        csrw!("mstatus", supervisor_mstatus.0);
        csrw!("mscratch", trap_frame_addr);
        csrw!("mtvec", trap_vector_ptr);
        csrw!("mie", interrupts_enable);
        // csrw!("mip", interrupts_enable);
        // csrw!("sie", interrupts_enable);
        // csrw!("sip", interrupts_enable);
        // loop {}
        // core::arch::asm!("
        // sd t1, 0(sp)
        // la t1, 4f
        // addi t1, t1, 4
        // csrw mepc, t1
        // ld t1, 0(sp)
        // mret
        // 4:
        // ");
    }
    abort();
    dbg!(csrr!("mscratch"));
    // crate::wfi();
}
