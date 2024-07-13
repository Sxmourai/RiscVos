// use std::env;
// use std::fs;
// use std::path::PathBuf;

fn main() {
    // let out_dir = PathBuf::from("target");
    // fs::write(out_dir.join("linker.ld"), include_bytes!("linker.ld")).unwrap();
    println!("cargo:rustc-link-arg-bins=--script=linker.ld", );
    println!("cargo:rerun-if-changed=linker.ld");
    println!("cargo:warn=test");

    println!("cargo:rerun-if-changed=build.rs");
}