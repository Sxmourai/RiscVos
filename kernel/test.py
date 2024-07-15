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
raw = ""
for test in os.listdir("tests"):
    with open("tests/"+test, "r") as f:
        raw_content = f.read()
        fnames = []
        for line in raw_content.splitlines():
            if "fn" in line:
                fname = line.strip().lstrip("pub ").lstrip("const ").lstrip("fn ")
                fname = fname[:fname.index("(")].strip()
                fnames.append(fname)
        raw += raw_content.replace("kernel::", "crate::") # We use kernel because rust-analyzer can find the library, or else he doesn't find anything and the test development process is longer

with open("target/compiled_tests.rs", "w") as f:
    f.write(raw)
    function_calls = ";\n    ".join(map(lambda fn: f"decorate_test(\"{fn}\", {fn})", fnames), )+";"
    f.write(f"""
pub fn test_all() {{
    {function_calls}
    close_qemu();
}}
""")
    
cmd(f"cargo b --profile {args.profile} {args.build_args} --features \"testing\"")
TARGET="riscv64gc-unknown-none-elf"
TARGET_DIR = "/".join((os.environ.get("CARGO_TARGET_DIR", "target"), TARGET))
KERNEL_FILE="/".join((TARGET_DIR,PROFILE_PATH,"kernel"))
# And append to QEMU: -drive if=none,format=raw,file=$(DRIVE),id=fat_disk -device virtio-blk-device,scsi=off,drive=fat_disk
import subprocess
cmd = subprocess.Popen(f"qemu-system-riscv64 -machine virt -smp {args.cpu_count} -m {args.mem_size} -nographic -serial mon:stdio -bios none -kernel {KERNEL_FILE} {args.qemu_args}".split(" "), stdout=subprocess.PIPE, stdin=subprocess.PIPE)
read = bytes(0)
try:
    while True:
        read += cmd.stdout.read(1)
        if read.decode(errors="ignore").strip().endswith("FLAG_EO_TESTS"):
            print("\r", end="") # Erase last FLAG_EO_TESTS print
            cmd.kill()
            break
        else:
            print(read[-1:].decode(errors="ignore"), end="")
except KeyboardInterrupt:pass