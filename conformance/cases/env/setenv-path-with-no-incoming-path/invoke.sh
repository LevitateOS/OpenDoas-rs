#!/bin/sh
exec env -i /usr/bin/doas -u root /usr/bin/python3 /conformance/fixtures/files/show_env.py PATH
