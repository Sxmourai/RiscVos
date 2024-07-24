from typing import List,Tuple, NamedTuple, Optional
import argparse
import os

TEST_DIR = "tests"

class Config(NamedTuple):
    args: argparse.Namespace
    target: str="riscv64gc-unknown-none-elf"
    
    def profile_path(self):
        profile_path = self.args.profile
        if self.args.profile == "dev":profile_path = "debug"
        return profile_path
        
    def target_dir(self):
        return "/".join((os.environ.get("CARGO_TARGET_DIR", "target"), self.target))
    def kernel_file(self):
        return "/".join((self.target_dir(),self.profile_path(),"kernel"))
    
    def from_args(args: argparse.Namespace):
        return Config(args)
_config = [None, ]
def config() -> Config:return _config[0]
def _strip_empty_cmd(cmd):return filter(lambda w: w != "", map(lambda x: x.strip(),cmd.split(" ")))

def create_parser(name="Kernel script", args: List[Tuple[list, dict]]=[]):
    parser = argparse.ArgumentParser(name)
    parser.add_argument("--profile", default="dev")
    parser.add_argument("--cpu-count", default="4")
    parser.add_argument("--mem-size", default="128M")
    parser.add_argument("--log-level", default="debug")
    parser.add_argument("--build-args", default="")
    parser.add_argument("--qemu-args", default="")
    parser.add_argument("-q", "--quiet", action=argparse.BooleanOptionalAction)
    for _args, _kwargs in args:
        parser.add_argument(*_args, **_kwargs)
    return parser

def parse_args(*args, **kwargs):
    args = create_parser(*args, **kwargs).parse_args()
    _config[0] = Config.from_args(args)
    return args


import subprocess
def build_kernel(args: argparse.ArgumentParser):
    if args.quiet:args.build_args += " -q "
    args.build_args += "--features log/max_level_"+args.log_level+" "
    c = list(_strip_empty_cmd(f"cargo b --profile {args.profile} {args.build_args}"))
    try:
        output = subprocess.check_output(c)
    except subprocess.CalledProcessError:
        print(" ".join(c), "failed.")
        exit(1)
    return output

def qemu_cmd(args: argparse.ArgumentParser):
    return subprocess.Popen(_strip_empty_cmd(f"""qemu-system-riscv64 
                        -machine virt -smp {args.cpu_count} -m {args.mem_size} 
                        -nographic -serial mon:stdio -bios none 
                        -drive if=none,format=raw,file=disk.hdd,id=fat_disk -device virtio-blk-device,scsi=off,drive=fat_disk
                        -kernel {config().kernel_file()}
                        {args.qemu_args}"""), stdout=subprocess.PIPE)



def handle_qemu_output(qemu: subprocess.Popen):
    import sys
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
                # print()
                # print(end="\r")
                print("\r"+" "*os.get_terminal_size()[0], end="\r")
                qemu.terminate()
                qemu.wait(1)
                # Delete qemu termination msg
                print("\033[1A\r"+" "*os.get_terminal_size()[0], end="\r")
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
                    try:
                        fname,path = subprocess.check_output(f"riscv64-unknown-elf-addr2line -e {config().kernel_file()} -f 0x{int(addr):x}".split(" ")).decode(errors='ignore').splitlines()
                    except PermissionError:print("You might not have binutils-riscv64-unknown-elf, we can't give backtrace info");exit(1)
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
