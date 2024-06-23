fn main() {
    println!("cargo:rustc-link-arg-bins=--script=linker.ld");
    println!("cargo:rustc-cfg=riscv");
    println!("cargo:rustc-cfg=riscv64");
}