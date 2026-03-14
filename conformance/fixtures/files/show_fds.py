#!/usr/bin/env python3
from pathlib import Path

fd_dir = Path("/proc/self/fd")
for entry in sorted(fd_dir.iterdir(), key=lambda item: int(item.name)):
    fd = int(entry.name)
    if fd <= 2:
        continue
    try:
        target = entry.readlink()
    except OSError:
        target = "<?>"
    print(f"{fd}:{target}")

