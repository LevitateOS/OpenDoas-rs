#!/bin/sh
exec env SHELL=/tmp/caller-shell doas -u bob -s

