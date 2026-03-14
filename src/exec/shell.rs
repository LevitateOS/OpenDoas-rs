use std::env;

use pwd_grp::Passwd;

pub fn selected_command(cmd: Option<String>, source_passwd: &Passwd) -> String {
    match cmd {
        Some(command) => command,
        None => env::var("SHELL")
            .ok()
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| source_passwd.shell.clone()),
    }
}
