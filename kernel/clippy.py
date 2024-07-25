import os
import sys

os.system(f"__CARGO_FIX_YOLO=1 cargo clippy {' '.join(sys.argv[1:])}")