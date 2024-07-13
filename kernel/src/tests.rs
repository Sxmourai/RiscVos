fn decorate_test(fname: &'static str, f: fn() -> bool) {
    crate::println!("=========Running {}=========", fname);
    if (f)() {
        crate::println!("\t   Success !");
    } else {
        crate::println!("\t   Failure !");
    }
}

#[cfg(feature="testing")]
include!("../target/compiled_tests.rs");
