#![no_std]
#![no_main]

use kernel::*;
core::arch::global_asm!(include_str!("boot.s"));

#[no_mangle] // Machine mode
extern "C" fn kinit() {
    kernel::logging::init();
    kernel::heap::init();
    #[cfg(feature = "testing")]
    kernel::tests::test_all();
    kernel::pmp::init(); // Needed by QEMU for mret, see https://stackoverflow.com/questions/69133848/risc-v-illegal-instruction-exception-when-switching-to-supervisor-mode
    kernel::traps::init(kmain as u64);
    unreachable!()
}

#[no_mangle]// Supervisor mode
extern "C" fn kmain() {
    kernel::plic::init();
    kernel::clint::init();
    // kernel::paging::init();
    info!("Done");
    #[cfg(feature = "testing")]
    kernel::tests::test_all();
    // unsafe {core::ptr::write_volatile(0x100 as *mut u8, 10)}
	loop {
        riscv::wfi()
    }
}
