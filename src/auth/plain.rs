use std::ffi::{CStr, CString};

use nix;
use pwd_grp;
use rpassword::read_password;
use shadow::Shadow;

use crate::platform::tty::{stdin_is_tty, write_prompt_to_tty};

#[link(name = "crypt")]
extern "C" {
    fn crypt(phrase: *const libc::c_char, setting: *const libc::c_char) -> *const libc::c_char;
}

pub fn challenge_user(passwd: &pwd_grp::Passwd) -> Result<(), &'static str> {
    if !stdin_is_tty() {
        return Err("a tty is required");
    }

    let hostname = nix::unistd::gethostname().expect("Failed to get hostname");
    let hostname = hostname.into_string().expect("Hostname is not valid UTF-8");
    let prompt = format!("\rdoas ({}@{}) password: ", &passwd.name, &hostname);
    write_prompt_to_tty(&prompt).map_err(|_| "a tty is required")?;
    let response = match read_password() {
        Ok(value) => value,
        Err(_) => return Err("Authentication failed"),
    };
    let mut hash = &passwd.passwd;
    let shadow;
    if hash == "x" {
        shadow = match Shadow::from_name(&passwd.name) {
            Some(value) => value,
            None => return Err("Authentication failed"),
        };
        hash = &shadow.password;
    }
    if verify_hash(hash, &response) {
        Ok(())
    } else {
        Err("Authentication failed")
    }
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
    use super::verify_hash;

    #[test]
    fn reject_hashes_with_nul_bytes() {
        assert!(!verify_hash("sha512\0hash", "password"));
    }

    #[test]
    fn reject_passwords_with_nul_bytes() {
        assert!(!verify_hash("sha512-hash", "pass\0word"));
    }
}
