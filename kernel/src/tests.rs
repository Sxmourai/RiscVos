fn decorate_test(fname: &'static str, f: fn() -> bool) {
    crate::println!("=========Running {}=========", fname);
    if (f)() {
        crate::println!("\t   Success !");
    } else {
        crate::println!("\t   Failure !");
    }
}

pub fn close_qemu() {
    crate::print!("FLAG_EO_TESTS"); // Interpreted in run.py and test.py to close qemu
}

#[cfg(feature="testing")]
include!("../target/compiled_tests.rs");
