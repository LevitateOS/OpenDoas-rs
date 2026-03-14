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
    unsafe {
        let hash = CString::new(hash).unwrap_unchecked();
        let response = CString::new(response).unwrap_unchecked();
        let result = crypt(response.as_ptr(), hash.as_ptr());
        let result = CStr::from_ptr(result).to_str().unwrap_unchecked();
        result == hash.to_str().unwrap_unchecked()
    }
}
