from os import system as cmd
import os
import argparse
parser = argparse.ArgumentParser("Kernel runner")
parser.add_argument("--profile", default="dev")
parser.add_argument("--cpu-count", default="4") # target/riscv64gc-unknown-none-elf/debug/kernel
parser.add_argument("--mem-size", default="128M")
parser.add_argument("--build-args", default="")
parser.add_argument("--qemu-args", default="")
args = parser.parse_args()
PROFILE_PATH = args.profile
if PROFILE_PATH == "dev":PROFILE_PATH = "debug"
cmd(f"cargo b --profile {args.profile} {args.build_args}")
TARGET="riscv64gc-unknown-none-elf"
TARGET_DIR = "/".join((os.environ.get("CARGO_TARGET_DIR", "target"), TARGET))
KERNEL_FILE="/".join((TARGET_DIR,PROFILE_PATH,"kernel"))
# When we will be able to read from disk
# DRIVE=fat32.raw
# And append to QEMU: -drive if=none,format=raw,file=$(DRIVE),id=fat_disk -device virtio-blk-device,scsi=off,drive=fat_disk
import subprocess
import sys
qemu = subprocess.Popen(map(lambda x: x.strip(), filter(lambda w: w.strip() != "", f"""qemu-system-riscv64 
                        -machine virt -smp {args.cpu_count} -m {args.mem_size} 
                        -nographic -serial mon:stdio -bios none 
                        -drive if=none,format=raw,file=disk.hdd,id=fat_disk -device virtio-blk-device,scsi=off,drive=fat_disk
                        -kernel {KERNEL_FILE} 
                        -msg timestamp=on,guest-name=on 
                        {args.qemu_args}""".split(" "))), stdout=subprocess.PIPE)
print(" ".join(qemu.args))
print()
read = ""
try:
    while True:
        read += qemu.stdout.read(1).decode(errors="ignore")
        if len(read)==0:break
        print(read[-1], end="")
        sys.stdout.flush()
        # decoded_read = read.decode(errors="ignore")
        # # print(read[-1:].decode(errors="ignore"), end="")
        # # print(decoded_read[-1:], end="")
        if read.endswith("FLAG_EO_TESTS") or read.endswith("QEMU: Terminated"):
            if qemu.stdin != None:
                qemu.stdin.close()
            qemu.stdout.close()
            print(end="\r")
            qemu.terminate()
            qemu.wait(1)
            break
        elif read.endswith("ERR_FROM_ADDR:"): # Pretty error messages from traps.rs
            parsed = qemu.stdout.read(1).decode(errors="ignore")
            addrs = [""]
            addr = 0
            while parsed[-1] != "\n":
                if parsed[-1] == ",":
                    addrs[addr] = int(addrs[addr])
                    addr += 1
                    addrs.append("")
                else:
                    addrs[addr] += parsed[-1]
                parsed += qemu.stdout.read(1).decode(errors="ignore")
            print()
            fnames = []
            paths = []
            for addr in addrs:
                fname,path = subprocess.check_output(f"riscv64-unknown-elf-addr2line -e {KERNEL_FILE} -f 0x{int(addr):x}".split(" ")).decode(errors='ignore').splitlines()
                fnames.append(fname)
                paths.append(path)
            with open("target/mangled.txt", "w") as f:
                f.write(",".join(fnames))
            demangled = subprocess.check_output(f"rustfilt -i target/mangled.txt".split(" ")).decode(errors="ignore").split(",")
            for i,(fname,path) in enumerate(zip(demangled, paths)):
                print(f"{i} -> {fname} at {path}")
                
            # print(demangled, paths, fnames)
        # else:
except KeyboardInterrupt:
    pass
finally:
    if qemu.stdin != None:
        qemu.stdin.close()
    qemu.stdout.close()
    print(end="\r")
    qemu.terminate()
    qemu.wait(1)

# Erase last FLAG_EO_TESTS print and other qemu stuff
sys.stdout.flush()
# print("\033[1A\r"+" "*os.get_terminal_size()[0])
# print()