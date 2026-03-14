#!/bin/sh
exec env TERM=vt100 DISPLAY=:1 doas -u root /usr/bin/python3 /conformance/fixtures/files/show_env.py DISPLAY TERM
