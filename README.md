# Introduction

This is a kernel for the RISC V architecture (currently without extensions, but we might add some in the future) (a.k.a. riscv64i)
We might also add a emulator and dissambler etc... A lot of good stuff !
Risc V seems really promising and I want to learn it !
I followed https://osblog.stephenmarz.com/ to get things started

## Requirements

- QEMU (Risc V) -> (qemu-system-misc on apt)
- Python
- Rust (nightly)
- Risc V binutils (for emulator)
If you want some debugging information:
- rustfilt (`cargo install rustfilt`)


## Running

To launch the kernel (make sure you have qemu and preferably on linux), run:
In kernel directory
```bash
python run.py
```

## Testing

In kernel directory
```bash
python test.py
```
Also if you want to use clippy, you should use it with --no-deps, so that it doesn't check in the tests folder or else it will cause some errors

## Clippy

In kernel directory
```bash
python clippy.py
```
Every argument you add after will be appended to clippy e.g.:
```bash
python clippy.py --allow-dirty
# Transforms into
__CARGO_FIX_YOLO=1 cargo clippy --allow-dirty
```

## Getting virt infos:
```bash
python3 run.py --qemu-args "-machine dumpdtb=out.dtb"
dtc --in-format dtb --out-format dts out.dtb --out out.dts
```


## Cool resources
- https://projectf.io