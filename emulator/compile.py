import os
import sys
os.system(f"riscv64-linux-gnu-as {sys.argv[1]}.s -o {sys.argv[1]}")
os.system(f"riscv64-linux-gnu-objcopy --strip-all -O binary {sys.argv[1]}")
