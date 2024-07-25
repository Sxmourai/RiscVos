import os,sys
file = sys.argv[1]
os.system(f"riscv64-linux-gnu-objdump {file} -D --target binary --architecture riscv:rv64")

