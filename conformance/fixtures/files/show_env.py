#!/usr/bin/env python3
import os
import sys

for key in sys.argv[1:]:
    if key in os.environ:
        print(f"{key}={os.environ[key]}")

