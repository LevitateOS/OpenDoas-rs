#!/bin/sh
exec env PATH=/custom/bin:/usr/bin doas -u root /usr/bin/env | sort

