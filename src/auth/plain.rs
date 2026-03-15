use std::ffi::{CStr, CString};

use nix;
use pwd_grp;
use rpassword::read_password;
use shadow::Shadow;

use crate::platform::tty::write_prompt_to_tty;

#[link(name = "crypt")]
extern "C" {
    fn crypt(phrase: *const libc::c_char, setting: *const libc::c_char) -> *const libc::c_char;
}

pub fn challenge_user(passwd: &pwd_grp::Passwd) -> Result<(), &'static str> {
    let hash = load_password_hash(passwd)?;

    let hostname = nix::unistd::gethostname().expect("Failed to get hostname");
    let hostname = hostname.into_string().expect("Hostname is not valid UTF-8");
    let prompt = format!("\rdoas ({}@{}) password: ", &passwd.name, &hostname);
    write_prompt_to_tty(&prompt).map_err(|_| "a tty is required")?;
    let response = match read_password() {
        Ok(value) => value,
        Err(_) => return Err("Authentication failed"),
    };
    if verify_hash(&hash, &response) {
        Ok(())
    } else {
        Err("Authentication failed")
    }
}

fn load_password_hash(passwd: &pwd_grp::Passwd) -> Result<String, &'static str> {
    if passwd.passwd == "x" {
        let shadow = Shadow::from_name(&passwd.name).ok_or("Authentication failed")?;
        return Ok(shadow.password);
    }

    if !passwd.passwd.starts_with('*') {
        return Err("Authentication failed");
    }

    Ok(passwd.passwd.clone())
}

pub fn verify_hash(hash: &str, response: &str) -> bool {
    let Ok(hash) = CString::new(hash) else {
        return false;
    };
    let Ok(response) = CString::new(response) else {
        return false;
    };

    unsafe {
        let result = crypt(response.as_ptr(), hash.as_ptr());
        if result.is_null() {
            return false;
        }

        CStr::from_ptr(result).to_bytes() == hash.as_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::{load_password_hash, verify_hash};

    #[test]
    fn rejects_non_shadowed_password_entries() {
        let passwd = pwd_grp::Passwd {
            name: String::from("alice"),
            passwd: String::from("!"),
            uid: 1000,
            gid: 1000,
            gecos: String::new(),
            dir: String::from("/home/alice"),
            shell: String::from("/bin/sh"),
            __non_exhaustive: Default::default(),
        };

        assert_eq!(load_password_hash(&passwd), Err("Authentication failed"));
    }

    #[test]
    fn accepts_star_prefixed_password_entries() {
        let passwd = pwd_grp::Passwd {
            name: String::from("alice"),
            passwd: String::from("*"),
            uid: 1000,
            gid: 1000,
            gecos: String::new(),
            dir: String::from("/home/alice"),
            shell: String::from("/bin/sh"),
            __non_exhaustive: Default::default(),
        };

        assert_eq!(load_password_hash(&passwd), Ok(String::from("*")));
    }

    #[test]
    fn reject_hashes_with_nul_bytes() {
        assert!(!verify_hash("sha512\0hash", "password"));
    }

    #[test]
    fn reject_passwords_with_nul_bytes() {
        assert!(!verify_hash("sha512-hash", "pass\0word"));
    }
}
