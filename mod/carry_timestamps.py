"""Keep Cargo's cache usable when promoting fresh Windows mixin stages."""
from pathlib import Path
import os
import sys

parent = Path(sys.argv[1]).resolve(strict=True)
old = Path(sys.argv[2]).resolve(strict=True)
new = Path(sys.argv[3]).resolve(strict=True)
for path in (old, new):
    if not path.is_relative_to(parent) or path == parent:
        raise SystemExit("Stage must stay inside its managed work folder")
count = 0
for target in new.rglob("*"):
    if not target.is_file() or target.is_symlink():
        continue
    source = old / target.relative_to(new)
    if source.is_file() and not source.is_symlink() and source.stat().st_size == target.stat().st_size:
        if source.read_bytes() == target.read_bytes():
            stamp = source.stat()
            os.utime(target, ns=(stamp.st_atime_ns, stamp.st_mtime_ns))
            count += 1
print(f"Kept timestamps of {count} unchanged files")
