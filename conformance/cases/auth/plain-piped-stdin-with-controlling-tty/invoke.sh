#!/bin/sh
exec /bin/sh -c "printf 'ignored\n' | doas -u root /usr/bin/id -u"
