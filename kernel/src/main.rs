#![no_std]
#![no_main]

use kernel::*;
core::arch::global_asm!(include_str!("boot.s"));

#[no_mangle] // Machine mode
extern "C" fn kinit() {
    kernel::logging::init();
    kernel::heap::init();
    kernel::pmp::init(); // Needed by QEMU for mret, see https://stackoverflow.com/questions/69133848/risc-v-illegal-instruction-exception-when-switching-to-supervisor-mode
    
    kernel::traps::init();
    // Supervisor mode
    kernel::plic::init();
    kernel::clint::init();
    kernel::paging::init();
    kernel::virtio::init();
    info!("Done");
    #[cfg(feature = "testing")]
    kernel::tests::test_all();
    kernel::main_loop()
}
