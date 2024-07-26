import os
def compile(file):
    out = f"target/{file}.o"
    os.system(f"riscv64-linux-gnu-as {file}.s -o target/{file}.o")
    os.system(f"riscv64-linux-gnu-objcopy --strip-all -O binary {out}")
    return out


if __name__ == "__main__":
    import sys
    try:
        os.mkdir("target")
    except FileExistsError:pass
    compile(sys.argv[1])
