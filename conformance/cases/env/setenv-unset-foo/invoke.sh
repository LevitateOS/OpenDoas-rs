#!/bin/sh
FOO=bar exec /usr/bin/doas /bin/sh -c 'if env | grep -q "^FOO="; then echo set; else echo unset; fi'
