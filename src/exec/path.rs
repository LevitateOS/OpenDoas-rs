use std::env;

pub const SAFE_PATH: &str = env!("SAFE_PATH");

pub fn safe_path() -> &'static str {
    SAFE_PATH
}

pub fn reset_process_path() {
    unsafe {
        env::set_var("PATH", SAFE_PATH);
    }
}

pub fn is_safe_path(path: &str) -> bool {
    path == SAFE_PATH
}
