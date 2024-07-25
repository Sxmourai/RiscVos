import script_lib
args = script_lib.parse_args(args=[[["--tests"], {"default": "all"}]])
args.build_args += '--features testing '

import os
raw = ""
fnames = []
for test in os.listdir(script_lib.TEST_DIR):
    with open(f"{script_lib.TEST_DIR}/{test}", "r") as f:
        raw_content = f.read()
        lines = ""
        for line in raw_content.splitlines(keepends=True):
            if line.startswith("#!"):continue
            if "fn" in line:
                fname = line.strip().lstrip("pub ").lstrip("const ").lstrip("fn ")
                fname = fname[:fname.index("(")].strip()
                fnames.append(fname)

            line = line.replace("kernel::", "crate::") # We use kernel because rust-analyzer can find the library, or else he doesn't find anything and the test development process is longer
            lines += line
        raw += lines

if args.tests != "all":
    tests = args.tests.split(",")
    fnames = filter(lambda fn: fn in tests, fnames)

with open("target/compiled_tests.rs", "w") as f:
    f.write(raw)
    tests_fns = ",".join(map(lambda fn: f"(\"{fn}\",{fn})", fnames))
    f.write(f"""
pub const TESTS_FNS: [(&'static str, fn()); {len(fnames)}] = [{tests_fns}];
""")

script_lib.build_kernel(args)
cmd = script_lib.qemu_cmd(args)
script_lib.handle_qemu_output(cmd)