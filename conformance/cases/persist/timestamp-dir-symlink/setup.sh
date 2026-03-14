#!/bin/sh
set -eu
rm -rf /run/doas
ln -s /tmp/doas-timestamps /run/doas

