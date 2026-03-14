use std::{
    fs::OpenOptions,
    io::{IsTerminal, Write},
};

pub fn stdin_is_tty() -> bool {
    std::io::stdin().is_terminal()
}

pub fn write_prompt_to_tty(prompt: &str) -> std::io::Result<()> {
    let mut tty = OpenOptions::new().write(true).open("/dev/tty")?;
    tty.write_all(prompt.as_bytes())?;
    tty.flush()
}

#[cfg(auth = "pam")]
pub fn current_tty_name() -> Option<String> {
    if let Ok(tty_path) = nix::unistd::ttyname(std::io::stdin()) {
        if let Ok(tty) = tty_path.strip_prefix("/dev/") {
            if let Some(tty) = tty.to_str() {
                return Some(tty.to_string());
            }
        }
    }
    None
}

#[cfg(not(auth = "pam"))]
pub fn current_tty_name() -> Option<String> {
    None
}
