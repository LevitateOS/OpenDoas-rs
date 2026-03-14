use std::ffi::CString;

use nix::unistd;
use pwd_grp::Passwd;

pub fn ensure_setuid_root() -> Result<(), String> {
    if unistd::geteuid().is_root() {
        Ok(())
    } else {
        Err(String::from("not installed setuid"))
    }
}

pub fn drop_to_real_uid() -> Result<(), String> {
    let uid = unistd::getuid();
    unistd::setresuid(uid, uid, uid).map_err(|err| format!("setresuid: {err}"))
}

pub fn switch_to_target(passwd_target: &Passwd) -> Result<(), String> {
    unistd::setresgid(
        passwd_target.gid.into(),
        passwd_target.gid.into(),
        passwd_target.gid.into(),
    )
    .map_err(|err| format!("setresgid: {err}"))?;
    let target_name =
        CString::new(passwd_target.name.clone()).map_err(|_| String::from("Invalid username"))?;
    unistd::initgroups(&target_name, passwd_target.gid.into())
        .map_err(|err| format!("initgroups: {err}"))?;
    unistd::setresuid(
        passwd_target.uid.into(),
        passwd_target.uid.into(),
        passwd_target.uid.into(),
    )
    .map_err(|err| format!("setresuid: {err}"))?;
    Ok(())
}
