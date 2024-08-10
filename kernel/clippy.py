import os
import sys

os.system(f"__CARGO_FIX_YOLO=1 cargo clippy --target riscv64gc-unknown-none-elf {' '.join(sys.argv[1:])}")