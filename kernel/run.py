from os import system as cmd
import os
import argparse
parser = argparse.ArgumentParser("Kernel runner")
parser.add_argument("--profile", default="dev")
parser.add_argument("--cpu-count", default="4")
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
cmd(f"qemu-system-riscv64 -machine virt -smp {args.cpu_count} -m {args.mem_size} -nographic -serial mon:stdio -bios none -kernel {KERNEL_FILE} {args.qemu_args}")