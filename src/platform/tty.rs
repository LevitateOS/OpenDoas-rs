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
    let tty = OpenOptions::new().read(true).open("/dev/tty").ok()?;
    let tty_path = nix::unistd::ttyname(&tty).ok()?;
    let tty = tty_path.strip_prefix("/dev/").ok()?;
    tty.to_str().map(|value| value.to_string())
}

#[cfg(not(auth = "pam"))]
pub fn current_tty_name() -> Option<String> {
    None
}
