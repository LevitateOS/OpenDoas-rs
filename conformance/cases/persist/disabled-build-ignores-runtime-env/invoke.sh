#!/bin/sh
exec env OPENDOAS_RS_TIMESTAMP=on doas -n -u root /usr/bin/id -u
