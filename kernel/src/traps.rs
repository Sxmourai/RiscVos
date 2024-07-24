// For cpu exceptions and interrupts

use riscv::PrivilegeLevel;

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
    todo!()
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
    SupervisorSoftware = 1<<1,
    MachineSoftware = 1<<3,
    SupervisorTimer = 1<<5,
    MachineTimer = 1<<7,
    SupervisorExternal = 1<<9,
    MachineExternal = 1<<11,
    CounterOverflow = 1<<13,
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
extern "C" fn mtrap() {
    let cause = csrr!("mcause", u64);
    let is_interrupt = cause & 1u64<<63 != 0;
    let id = cause & 0xFF;
    let mtval = csrr!("mtval", u64);
    if is_interrupt {
        match id {
            1 => {println!("Supervisor software interrupt")},
            3 => {println!("Machine software interrupt")},
            5 => {println!("Supervisor timer interrupt")},
            7 => {clint::handle_int()},
            9 => {println!("Supervisor external interrupt")},
            11 => {
                use plic::PLIC;
                if let Some(int) = PLIC.claim_int() {
                    match int {
                        10 => logging::handle_int(),
                        1..=8 => {virtio::handle_int(int)},
                        _ => todo!("Support this interrupt: {} !", int)
                    }
                    unsafe { PLIC.eoi(int) };
                } else {
                    warn!("Spurious interrupt ?!");
                }
            },
            13 => {println!("Counter overflow interrupt")},
            ..16 => {println!("Custom interrupt !")},
            _ => {println!("Interrupt cause: {:b}", id);},
        }
    } else {
        println!("Exception {} from mode: {}", id, riscv::MSTATUS::read().mpp());
        match id {
            0 => {println!("Instruction address misaligned: {}", mtval)},
            1 => {println!("Instruction access fault: {}", mtval)},
            2 => {println!("Illegal instruction: {}", csrr!("mepc")); unsafe{csrw!("mepc", csrr!("mepc")+4)}},
            3 => {println!("Breakpoint: {}", mtval)},
            4 => {println!("Load address misaligned: {}", mtval)},
            5 => {println!("Load access fault: {}", mtval)}, // Fail pmp check
            6 => {println!("Store/AMO address misaligned: {}", mtval)},
            7 => {println!("Store/AMO access fault: {}", mtval)},
            8 => {println!("Environment call from U-mode: {}", mtval)},
            9 => {println!("Environment call from S-mode: {}", mtval)},
            11 => {println!("Environment call from M-mode: {}", mtval)},
            12 => {println!("Instruction page fault: {}", mtval)},
            13 => {println!("Load page fault: {}", mtval);unsafe{map!(mtval, mtval)};panic!()}, // Fail page check
            15 => {println!("Store/AMO page fault: {}", mtval)},
            18 => {println!("Software check: {}", mtval)},
            19 => {println!("Hardware error: {}", mtval)},
            _ => {dbg!(cause, mtval);},
        }
        tests::log_err()
        
    }
}

#[no_mangle]
extern "C" fn strap() {
    let cause = csrr!("scause", u64);
    let is_interrupt = cause & 1u64<<63 != 0;
    let id = cause & 0xFF;
    // crate::print!("Trap: {}\t", id);
    let mtval = csrr!("stval", u64);
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
            0 => {todo!("Instruction address misaligned: {}", mtval)},
            1 => {todo!("Instruction access fault: {}", mtval)},
            2 => {todo!("Illegal instruction: {}", mtval)},
            3 => {todo!("Breakpoint: {}", mtval)},
            4 => {todo!("Load address misaligned: {}", mtval)},
            5 => {todo!("Load access fault: {}", mtval)},
            6 => {todo!("Store/AMO address misaligned: {}", mtval)},
            7 => {todo!("Store/AMO access fault: {}", mtval)},
            8 => {todo!("Environment call from U-mode: {}", mtval)},
            9 => {todo!("Environment call from S-mode: {}", mtval)},
            11 => {todo!("Environment call from M-mode: {}", mtval)},
            12 => {todo!("Instruction page fault: {}", mtval)},
            13 => {todo!("Load page fault: {}", mtval)},
            15 => {todo!("Store/AMO page fault: {}", mtval)},
            18 => {todo!("Software check: {}", mtval)},
            19 => {todo!("Hardware error: {}", mtval)},
            _ => {dbg!(cause, mtval);},
        }
    }
}

extern "C" {
    pub fn s_trap_vector();
    pub fn m_trap_vector();
}
#[no_mangle]
pub extern "C" fn abort() {
    dbg!(csrr!("mcause"), csrr!("mtval"), csrr!("mscratch"));
    dbg!(csrr!("mepc"), csrr!("mtvec"));
    let mut res: u64;
    unsafe{core::arch::asm!("mv {}, t5", out(reg) res)};
    dbg!(res);
    unsafe{core::arch::asm!("mv {}, sp", out(reg) res)};
    dbg!(res);
    println!("Aborting...");
    // loop {}
} 

core::arch::global_asm!(" # Thx core-os
.option norvc

.align 4
.global s_trap_vector
.global m_trap_vector

s_trap_vector:
// ------------ STORE
// Space for registers
addi sp, sp, -256

sd ra, 0(sp)
sd sp, 8(sp)
sd gp, 16(sp)
sd tp, 24(sp)
sd t0, 32(sp)
sd t1, 40(sp)
sd t2, 48(sp)
sd s0, 56(sp)
sd s1, 64(sp)
sd a0, 72(sp)
sd a1, 80(sp)
sd a2, 88(sp)
sd a3, 96(sp)
sd a4, 104(sp)
sd a5, 112(sp)
sd a6, 120(sp)
sd a7, 128(sp)
sd s2, 136(sp)
sd s3, 144(sp)
sd s4, 152(sp)
sd s5, 160(sp)
sd s6, 168(sp)
sd s7, 176(sp)
sd s8, 184(sp)
sd s9, 192(sp)
sd s10,200(sp)
sd s11,208(sp)
sd t3, 216(sp)
sd t4, 224(sp)
sd t5, 232(sp)
sd t6, 240(sp)
// ------  END STORE
call strap
// ------------ LOAD
ld ra, 0(sp)
ld sp, 8(sp)
ld gp, 16(sp)
// tp
ld t0, 32(sp)
ld t1, 40(sp)
ld t2, 48(sp)
ld s0, 56(sp)
ld s1, 64(sp)
ld a0, 72(sp)
ld a1, 80(sp)
ld a2, 88(sp)
ld a3, 96(sp)
ld a4, 104(sp)
ld a5, 112(sp)
ld a6, 120(sp)
ld a7, 128(sp)
ld s2, 136(sp)
ld s3, 144(sp)
ld s4, 152(sp)
ld s5, 160(sp)
ld s6, 168(sp)
ld s7, 176(sp)
ld s8, 184(sp)
ld s9, 192(sp)
ld s10, 200(sp)
ld s11, 208(sp)
ld t3, 216(sp)
ld t4, 224(sp)
ld t5, 232(sp)
ld t6, 240(sp)

addi sp, sp, 256
// ---------- END LOAD
sret


.align 4
m_trap_vector:
// ------------ STORE
// Space for registers
addi sp, sp, -256

sd ra, 0(sp)
sd sp, 8(sp)
sd gp, 16(sp)
sd tp, 24(sp)
sd t0, 32(sp)
sd t1, 40(sp)
sd t2, 48(sp)
sd s0, 56(sp)
sd s1, 64(sp)
sd a0, 72(sp)
sd a1, 80(sp)
sd a2, 88(sp)
sd a3, 96(sp)
sd a4, 104(sp)
sd a5, 112(sp)
sd a6, 120(sp)
sd a7, 128(sp)
sd s2, 136(sp)
sd s3, 144(sp)
sd s4, 152(sp)
sd s5, 160(sp)
sd s6, 168(sp)
sd s7, 176(sp)
sd s8, 184(sp)
sd s9, 192(sp)
sd s10,200(sp)
sd s11,208(sp)
sd t3, 216(sp)
sd t4, 224(sp)
sd t5, 232(sp)
sd t6, 240(sp)
// ------  END STORE
call mtrap
// ------------ LOAD
ld ra, 0(sp)
ld sp, 8(sp)
ld gp, 16(sp)
// tp
ld t0, 32(sp)
ld t1, 40(sp)
ld t2, 48(sp)
ld s0, 56(sp)
ld s1, 64(sp)
ld a0, 72(sp)
ld a1, 80(sp)
ld a2, 88(sp)
ld a3, 96(sp)
ld a4, 104(sp)
ld a5, 112(sp)
ld a6, 120(sp)
ld a7, 128(sp)
ld s2, 136(sp)
ld s3, 144(sp)
ld s4, 152(sp)
ld s5, 160(sp)
ld s6, 168(sp)
ld s7, 176(sp)
ld s8, 184(sp)
ld s9, 192(sp)
ld s10, 200(sp)
ld s11, 208(sp)
ld t3, 216(sp)
ld t4, 224(sp)
ld t5, 232(sp)
ld t6, 240(sp)

addi sp, sp, 256
// ---------- END LOAD
mret
");


pub fn init(callback: usize) {
    info!("Initialising traps...");
    let mut supervisor_mstatus = riscv::MSTATUS(PrivilegeLevel::supervisor());
    supervisor_mstatus.set_mpie(true); // I think mpie should be set anyway, because we can't have spie and not mpie (see ISA / doc)
    supervisor_mstatus.set_spie(true);
    supervisor_mstatus.set_mie(true);
    supervisor_mstatus.set_sie(true);

    unsafe{
        csrw!("mstatus", supervisor_mstatus.0);
        csrw!("satp", 0);
        // Delegate all interrupts to supervisor mode (so that we only have 1 interrupt handler)
        // csrw!("medeleg", 0xffff);
        // csrw!("mideleg", 0xffff);
        csrw!("mtvec", m_trap_vector as usize & !(0b11));
        // use Interrupts as Int;(Int::SupervisorTimer as u64) | (Int::SupervisorSoftware as u64) | (Int::SupervisorExternal as u64) |
        // (Int::MachineTimer as u64) | (Int::MachineSoftware as u64) | (Int::MachineExternal as u64)
        csrw!("mie", 0xFFFF);
        csrw!("sie", 0xFFFF);
        csrw!("stvec", s_trap_vector as usize & !(0b11));
        csrw!("mepc", callback);
        core::arch::asm!("
        // csrw mepc, 
        mret", options(noreturn));
    }
}
