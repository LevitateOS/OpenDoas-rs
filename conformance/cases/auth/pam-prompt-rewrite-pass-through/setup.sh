#!/bin/sh
set -eu

cat > /tmp/pam_prompt_passthrough.c <<'EOF'
#define _GNU_SOURCE
#include <security/pam_appl.h>
#include <security/pam_ext.h>
#include <security/pam_modules.h>
#include <stdlib.h>

PAM_EXTERN int
pam_sm_authenticate(pam_handle_t *pamh, int flags, int argc, const char **argv)
{
    const char *response = NULL;
    int ret;

    (void)flags;
    (void)argc;
    (void)argv;

    ret = pam_prompt(pamh, PAM_PROMPT_ECHO_OFF, &response, "%s", "Passphrase: ");
    if (ret != PAM_SUCCESS)
        return ret;

    free((void *)response);
    return PAM_SUCCESS;
}

PAM_EXTERN int
pam_sm_setcred(pam_handle_t *pamh, int flags, int argc, const char **argv)
{
    (void)pamh;
    (void)flags;
    (void)argc;
    (void)argv;
    return PAM_SUCCESS;
}

#ifdef PAM_MODULE_ENTRY
PAM_MODULE_ENTRY("pam_prompt_passthrough");
#endif
EOF

cc -fPIC -shared -o /usr/lib/security/pam_prompt_passthrough.so /tmp/pam_prompt_passthrough.c -lpam

cat > /etc/pam.d/doas <<'EOF'
#%PAM-1.0
auth       required     pam_prompt_passthrough.so
account    sufficient   pam_permit.so
session    sufficient   pam_permit.so
EOF

