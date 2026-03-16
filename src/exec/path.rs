use std::{env, ffi::OsStr};

pub const SAFE_PATH: &str = env!("SAFE_PATH");

pub fn safe_path() -> &'static str {
    SAFE_PATH
}

pub fn reset_process_path() {
    set_process_path(OsStr::new(SAFE_PATH));
}

pub fn set_process_path(path: &OsStr) {
    unsafe {
        env::set_var("PATH", path);
    }
}

pub fn is_safe_path(path: &str) -> bool {
    path == SAFE_PATH
}
