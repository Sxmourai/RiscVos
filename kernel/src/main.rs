#![no_std]
#![no_main]

use kernel::{csrr, dbg, print, println};

core::arch::global_asm!(include_str!("boot.s"));
unsafe fn assert_mstatus() {
    let mstatus = csrr!("mstatus");
    unsafe { kernel::paging::enter_mode(kernel::paging::PrivilegeLevel::Machine) }
    if mstatus != csrr!("mstatus") {
        dbg!(mstatus, csrr!("mstatus"));
    }
} 

#[no_mangle]
extern "C" fn kmain() {
    unsafe{kernel::console::STDIO_UART.init()};
    unsafe {assert_mstatus()};
    kernel::heap::init();
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

// Could be unsafe ? Because it could stop os if no interrupts ?
fn wfi() {
    unsafe {
        core::arch::asm!("wfi"); // "hlt" in x86
    }
}

pub fn spin_loop() -> ! {
    loop {wfi()}
}

// https://github.com/sgmarz/osblog/blob/master/risc_v/src/main.rs
#[no_mangle]
pub extern "C" fn abort() -> ! {
	spin_loop()
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
	print!("Aborting: ");
	if let Some(p) = info.location() {
		println!(
					"file {}:{} - {}",
					p.file(),
					p.line(),
					info.message()
		);
	}
	else {
		println!("no information available.");
	}
    spin_loop()
}