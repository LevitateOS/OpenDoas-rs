#!/usr/bin/env python3
import os
import signal
import socket
import sys

if len(sys.argv) != 3:
    raise SystemExit("usage: syslog_capture.py <socket-path> <output-path>")

socket_path = sys.argv[1]
output_path = sys.argv[2]
running = True

def stop(_signum, _frame):
    global running
    running = False

signal.signal(signal.SIGTERM, stop)
signal.signal(signal.SIGINT, stop)

try:
    os.unlink(socket_path)
except FileNotFoundError:
    pass

sock = socket.socket(socket.AF_UNIX, socket.SOCK_DGRAM)
sock.bind(socket_path)
sock.settimeout(0.2)

with open(output_path, "w", encoding="utf-8") as out:
    while running:
        try:
            data = sock.recv(65535)
        except socket.timeout:
            continue
        out.write(data.decode("utf-8", "replace"))
        if not data.endswith(b"\n"):
            out.write("\n")
        out.flush()

sock.close()
try:
    os.unlink(socket_path)
except FileNotFoundError:
    pass

