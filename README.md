# Introduction

This is a kernel for the RISC V architecture (currently without extensions, but we might add some in the future) (a.k.a. riscv64i)
We might also add a emulator and dissambler etc... A lot of good stuff !
Risc V seems really promising and I want to learn it !

## Requirements

- QEMU (Risc V) -> (qemu-system-misc on apt)
- Python
- Rust (nightly)

## Running

To launch the kernel (make sure you have qemu and preferably on linux), run:

```bash
python main.py kernel
```
