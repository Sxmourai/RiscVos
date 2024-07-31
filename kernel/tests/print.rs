#![no_std]
#![feature(custom_inner_attributes)]
#![clippy::allow(all)]

// The raw contents of this file will be placed in src/tests.rs when running test.py
fn print_simple() {
    kernel::println!("Hello world !");
}
fn print_2() {
    kernel::println!("Hello world ! 🥰");
}