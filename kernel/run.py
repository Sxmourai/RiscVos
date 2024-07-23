import script_lib as sl
args = sl.parse_args()
sl.build_kernel(args)
qemu = sl.qemu_cmd(args)
sl.handle_qemu_output(qemu)
