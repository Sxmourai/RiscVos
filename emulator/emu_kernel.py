import os
os.system(f"cd ../kernel && cargo b")
os.system(f"riscv64-linux-gnu-objcopy --strip-all -O binary ../kernel/target/riscv64gc-unknown-none-elf/debug/kernel target/rust_kernel")
os.system(f"cargo r target/rust_kernel")
