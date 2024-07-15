#![no_std]
#![no_main]

use kernel::*;
core::arch::global_asm!(include_str!("boot.s"));

#[no_mangle]
extern "C" fn kmain() {
    unsafe{kernel::console::STDIO_UART.init()};
    unsafe {assert_mstatus()};
    kernel::heap::init();
    #[cfg(feature = "testing")]
    kernel::tests::test_all();
    kernel::traps::init();
    kernel::paging::init();
    kernel::plic::init();
    println!("Booting: Risc-V os v0.0.0 ...");
    #[cfg(feature = "testing")]
    kernel::tests::test_all();
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
                    print!("{}", input_char);
                    print!("{}", input_char);
                    unsafe{kernel::console::STDIO_UART.write_chr(input_char)};
                }
            }
        }
        // wfi()
        // core::hint::spin_loop()
    }
    // spin_loop()
}
