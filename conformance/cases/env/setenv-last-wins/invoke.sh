#!/bin/sh
exec doas -u root /usr/bin/python3 /conformance/fixtures/files/show_env.py FOO BAR
