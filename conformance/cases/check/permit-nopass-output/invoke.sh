#!/bin/sh
exec /usr/bin/doas -C /case/doas.conf -u root /usr/bin/id -u
