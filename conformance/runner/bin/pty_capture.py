#!/usr/bin/env python3
import os
import pty
import select
import subprocess
import sys

if len(sys.argv) < 4:
    raise SystemExit("usage: pty_capture.py <output-file> <stdin-file> <cmd> [args...]")

output_file = sys.argv[1]
stdin_file = sys.argv[2]
cmd = sys.argv[3:]

master_fd, slave_fd = pty.openpty()
stdin_data = b""
if stdin_file != "/dev/null":
    with open(stdin_file, "rb") as handle:
        stdin_data = handle.read()

proc = subprocess.Popen(cmd, stdin=slave_fd, stdout=slave_fd, stderr=slave_fd, close_fds=True)
os.close(slave_fd)

if stdin_data:
    os.write(master_fd, stdin_data)

chunks = []
while True:
    readable, _, _ = select.select([master_fd], [], [], 0.1)
    if master_fd in readable:
        try:
            data = os.read(master_fd, 4096)
        except OSError:
            break
        if not data:
            break
        chunks.append(data)
    if proc.poll() is not None and not readable:
        try:
            data = os.read(master_fd, 4096)
            if data:
                chunks.append(data)
        except OSError:
            pass
        break

os.close(master_fd)
exit_code = proc.wait()

with open(output_file, "wb") as handle:
    for chunk in chunks:
        handle.write(chunk)

raise SystemExit(exit_code)

