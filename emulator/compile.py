import os
def compile(file):
    os.system(f"riscv64-linux-gnu-as {file}.s -o target/{file}.o")
    os.system(f"riscv64-linux-gnu-objcopy --strip-all -O binary target/{file}.o")


if __name__ == "__main__":
    import sys
    try:
        os.mkdir("target")
    except FileExistsError:pass
    compile(sys.argv[1])
