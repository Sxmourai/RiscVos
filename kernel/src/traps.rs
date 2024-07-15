// For cpu exceptions and interrupts

use crate::{csrr, dbg, println};

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
    dbg!(addr);
    unsafe {
        save_registers!(addr, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31);
    }
}

#[no_mangle]
unsafe extern "C" fn trap_vector() {
    let cause = csrr!("mcause", u64);
    dbg!();
    let is_interrupt = cause & 1u64<<63 != 0;
    let id = cause & 0xFF;
    if is_interrupt {
        match id {
            1 => {},
            _ => {println!("Interrupt cause: {:b}", id);},
        }
    } else {
        println!("CPU Exception: {:b}", id);
    }

    // We can write at address 0x100, just after VIRT_DEBUG in QEMU and before VIRT_MROM
    // println!("{:b}", csrr!("mtval", u64));
    // for i in 0..1472762 {}
    // save_context(0x100);
    // Is interrupt is like is asynchronous
    // (cpu exceptions are considered to be synchronous, because they raise with the current instruction, )
    // whereas interrupts can be caused anywhere

    // core::arch::asm!("mret");
    // let table = TrapFrame::new();
    // let table_addr = 
//     core::arch::asm!("
//     csrrw	t6, mscratch, t6 // Swap t6 and mscratch

// // Save all registers apart from t6 (last register x31)
// .set 	i, 1
// .rept	30
// 	sd x%i,  
// 	.set	i, i+1
// .endr
// // Save t6 
// mv		t5, t6
// csrr	t6, mscratch
// save_gp 31, t5

// // Restore the kernel trap frame into mscratch
// csrw mscratch, t5

// csrr a0, mepc
// csrr a1, mtval
// csrr a2, mcause
// csrr a3, mhartid
// csrr a4, mstatus
// mv   a5, t5
// ld   sp, 520(a5)

// ");
//     _trap_vector();
//     core::arch::asm!("
// // Rust function sets return address in a0
// csrw mepc, a0
// // load the trap frame back into t6
// csrr t6, mscratch

// # Restore all GP registers
// .set	i, 1
// .rept	31
// 	load_gp %i
// 	.set	i, i+1
// .endr

// # Since we ran this loop 31 times starting with i = 1,
// # the last one loaded t6 back to its original value.
// mret
// ");
}


pub fn init() {
    println!("Initialising traps...");
    let trap_vector_ptr = trap_vector as usize & !(0b11);
    unsafe{core::arch::asm!("csrw mtvec, {}
    csrw stvec, {}
    csrw mie, {}
    csrw mip, {}", in(reg) trap_vector_ptr, in(reg) 0, in(reg) 0xaaa, in(reg) 0xaaa)};
}
