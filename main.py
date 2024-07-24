import argparse
import subprocess
import sys
import os

parser = argparse.ArgumentParser(
                    prog='Risc V launcher utility',
                    description='Helps you launch / compile the different programs available (currently only kernel)',)

parser.add_argument("launch", default="kernel")
parser.add_argument("--release", action="store_true")
parser.add_argument("--target", default="riscv64gc-unknown-linux-gnu", required=False)
s = 0
for act in parser._actions:
    if act.dest != "help":
        s += 1
args = parser.parse_args(sys.argv[1:1+s])
def cmd(c):
    res = os.system(c, ) # capture_output=True
    if res != 0:
        exit(res)
    # else:
    #     print(res)
    return res
cmd_spec_args = sys.argv[1+s:]
profile = "--release" if args.release else ""
path_profile = "release" if args.release else "debug"
if args.launch.lower() == "kernel":
    os.chdir("kernel")
    parser = argparse.ArgumentParser("Kernel compiler")
    parser.add_argument("--debugger", action="store_true")
    spec_args = parser.parse_args(cmd_spec_args)
    cmd(f"cargo b {profile}")
    cmd(f"riscv64-linux-gnu-objcopy -O binary --only-section=.text target/{args.target}/{path_profile}/kernel target/kernel")
    # get raw asm: riscv64-linux-gnu-objdump -b binary -m riscv -D target/kernel
    debugger = "-s -S" if spec_args.debugger else ""
    cmd(f"qemu-system-riscv64 -machine virt -bios none -serial stdio -display none {debugger} -kernel target/{args.target}/{path_profile}/kernel")
    
if args.launch.lower() == "disasm":
    os.chdir("kernel")
    def cmd(c):
        res = os.system(c)
        if res != 0:
            exit(res)
        return res
    cmd(f"riscv64-linux-gnu-objdump -d target/{args.target}/{path_profile}/kernel")