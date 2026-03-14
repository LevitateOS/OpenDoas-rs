use pwd_grp::{self, Passwd};

pub fn current_passwd() -> Result<Passwd, String> {
    pwd_grp::getpwuid(pwd_grp::getuid())
        .map_err(|err| err.to_string())?
        .ok_or_else(|| String::from("no passwd entry for self"))
}

pub fn parse_target_uid(spec: &str) -> Result<u32, String> {
    if let Some(passwd) = pwd_grp::getpwnam(spec).map_err(|err| err.to_string())? {
        return Ok(passwd.uid.into());
    }

    let Ok(uid) = spec.parse::<u32>() else {
        return Err(String::from("unknown user"));
    };

    pwd_grp::getpwuid(uid.into())
        .map_err(|err| err.to_string())?
        .map(|passwd| passwd.uid.into())
        .ok_or_else(|| String::from("unknown user"))
}

pub fn target_passwd(uid: u32) -> Result<Passwd, String> {
    pwd_grp::getpwuid(uid.into())
        .map_err(|err| err.to_string())?
        .ok_or_else(|| String::from("no passwd entry for target"))
}
