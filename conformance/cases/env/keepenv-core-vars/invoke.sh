#!/bin/sh
exec env \
  HOME=/bad/home \
  LOGNAME=badlog \
  USER=baduser \
  SHELL=/bin/false \
  DOAS_USER=bad-doas-user \
  PATH=/bad/path \
  DISPLAY=:99 \
  TERM=vt100 \
  doas -u bob /usr/bin/python3 /conformance/fixtures/files/show_env.py \
    HOME LOGNAME USER SHELL DOAS_USER PATH DISPLAY TERM
