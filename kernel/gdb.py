import os,sys
from script_lib import * 

if sys.argv[1] == "qemu":
    run(fake_args(sys.argv[2:]+["--qemu-args", "-s -S"]))
if sys.argv[1] == "gdb":
    os.system(f'gdb-multiarch -ex "set architecture riscv:rv64" -ex "target remote localhost:1234" {config().kernel_file()} -q')