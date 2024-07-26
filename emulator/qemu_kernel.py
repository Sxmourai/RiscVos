import os,sys
from compile import compile 
out = compile(sys.argv[1])

os.system(f"qemu-system-riscv64 -kernel {out} -machine virt -nographic")