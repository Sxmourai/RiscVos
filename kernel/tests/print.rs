#![no_std]
#![feature(custom_inner_attributes)]
#![clippy::allow(all)]

// The raw contents of this file will be placed in src/tests.rs when running test.py
fn print_simple() -> bool {
    kernel::println!("Hello world !");
    true
}
fn print_2() -> bool {
    kernel::println!("Hello world ! 🥰");
    true
}