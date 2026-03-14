#!/bin/sh
exec env -u MISSING doas -u root /usr/bin/python3 /conformance/fixtures/files/show_env.py FOO
