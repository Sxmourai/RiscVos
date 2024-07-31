use heap::MAIN_HEAP_ALLOCATOR;

use crate::*;

#[cfg(feature="testing")]
include!("../target/compiled_tests.rs");
#[cfg(feature="testing")]
pub fn test_all() {
    for (test_name, test) in TESTS_FNS {
        crate::println!("=========Running {}=========", test_name);
        (test)();
        crate::println!("\t   Success !");
    }
}


pub fn close_qemu() {
    crate::print!("FLAG_EO_TESTS"); // Interpreted in run.py and test.py to close qemu
}

pub fn log_err() {
    let sp = regr!("sp")+0x100;
    let stack = unsafe { core::slice::from_raw_parts(sp as *const usize, 100) };
    // dbg!(stack);
    print!("ERR_FROM_ADDR:{}", csrr!("mepc"));
    for val in stack {
        if *val >= 0x8000_0000 && *val <= riscv::stack_start() {
            print!(",{}", val);
        }
    }
    println!("");
    if unsafe { MAIN_HEAP_ALLOCATOR.idx() } > 10000 {
        dbg!(unsafe { MAIN_HEAP_ALLOCATOR.idx() });
    }
}


pub static mut PANIC_CALLBACK: Option<fn()> = None;
#[panic_handler]
fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
    log::error!("PANIC: ");
    if let Some(loc) = info.location() {
        crate::print!("at {}:{}:{} ", loc.file(), loc.line(), loc.column());
    };
    if let Some(msg) = info.message().as_str() {
        crate::println!("{}", msg);
    } else {
        crate::println!("{}", alloc::string::ToString::to_string(&info.message()));
    }
    log_err();
    unsafe {
        if let Some(f) = PANIC_CALLBACK {
            log::warn!("Calling callback at {:?}", f);
            f();
            PANIC_CALLBACK = None;
            log::warn!("Panic callback returned !");
        }
    }
    loop {riscv::wfi()}
}