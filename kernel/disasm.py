import run
import os
os.system(f"riscv64-linux-gnu-objdump -d -j .text {run.sl.config().kernel_file()}")