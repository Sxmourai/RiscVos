#![no_std]
#![no_main]

use kernel::*;
core::arch::global_asm!(include_str!("boot.s"));

#[no_mangle] // Machine mode
extern "C" fn kinit() {
    unsafe{kernel::console::STDIO_UART.init()};
    unsafe {riscv::assert_mstatus()};
    kernel::heap::init();
    #[cfg(feature = "testing")]
    kernel::tests::test_all();
    kernel::pmp::init(); // Needed by QEMU for mret, see https://stackoverflow.com/questions/69133848/risc-v-illegal-instruction-exception-when-switching-to-supervisor-mode
    kernel::traps::init(kmain as u64);
    println!("Test");
}

#[no_mangle]// Supervisor mode
extern "C" fn kmain() {
    // println!("Supervisor m");
    // kernel::paging::init();
    // kernel::plic::init();
    println!("Booting: Risc-V os v0.0.0 ...");
    #[cfg(feature = "testing")]
    kernel::tests::test_all();
    // unsafe {core::ptr::write_volatile(0x100 as *mut u8, 10)}
	loop {
        let input_char = unsafe{kernel::console::STDIO_UART.read_chr()};
        if let Some(input_char) = input_char {
            match input_char {
                127 => { // Backspace
                    print!("{}{}{}", 8 as char, ' ', 8 as char);
                },
                10 | 13 => {
                    println!();
                },
                3 => {// From what i've seen it's CTRL+C
                    kernel::tests::close_qemu()
                },
                _ => {
                    print!("{}", input_char as char);
                }
            }
        }
        // riscv::wfi()
        // core::hint::spin_loop()
    }
    // spin_loop()
}
