#![no_std]
#![no_main]

core::arch::global_asm!("
.section .init

.option norvc

.type start, @function
.global start
start:
	
    .cfi_startproc
    
.option push
.option norelax
	la gp, global_pointer
.option pop
	
	/* Reset satp */
	csrw satp, zero
	
	/* Setup stack */
	la sp, stack_top
	
	/* Clear the BSS section */
	la t5, bss_start
	la t6, bss_end
bss_clear:
	sd zero, (t5)
	addi t5, t5, 8
	bltu t5, t6, bss_clear
	
	la t0, kmain
	csrw mepc, t0
	
	/* Jump to kernel! */
	tail kmain
	
	.cfi_endproc

    .section .rodata

    debug_string:
            .string \"Hello world\n\"

.end
");

#[no_mangle]
pub unsafe extern "C" fn kmain(a0: usize, a1: usize, a2: usize) -> ! {
    unsafe { core::ptr::write_volatile(0x10000000 as *mut u8, b'h') }
    unsafe { core::ptr::write_volatile(0x10000000 as *mut u8, b'h') }
    unsafe { core::ptr::write_volatile(0x10000000 as *mut u8, b'h') }
    unsafe { core::ptr::write_volatile(0x10000000 as *mut u8, b'h') }
    
    loop {}
}


#[panic_handler]
fn _panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}