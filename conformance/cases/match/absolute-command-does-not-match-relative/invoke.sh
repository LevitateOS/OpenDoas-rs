#!/bin/sh
exec doas -C /etc/doas.conf -u root id -u
