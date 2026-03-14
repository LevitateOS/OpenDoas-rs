//! No-auth backend for nopass-only builds.

use crate::RuleOpts;

pub const AUTH_NONE_MESSAGE: &str =
    "This command requires authentication but this version of OpenDoas-rs was built without any authentication methods!";

pub fn ensure_nopass(rule_opts: &RuleOpts) -> Result<(), &'static str> {
    if rule_opts.nopass {
        Ok(())
    } else {
        Err(AUTH_NONE_MESSAGE)
    }
}
