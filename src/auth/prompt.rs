use std::{borrow::Cow, ffi::OsStr};

fn hostname_label(hostname: &OsStr) -> Cow<'_, str> {
    hostname.to_string_lossy()
}

pub fn password_prompt_for_hostname(username: &str, hostname: &OsStr) -> String {
    format!(
        "\rdoas ({}@{}) password: ",
        username,
        hostname_label(hostname)
    )
}

pub fn password_prompt(username: &str) -> String {
    let hostname = nix::unistd::gethostname().ok();
    let label = hostname
        .as_deref()
        .map(hostname_label)
        .unwrap_or_else(|| Cow::Borrowed("?"));

    format!("\rdoas ({}@{}) password: ", username, label)
}

#[cfg(test)]
mod tests {
    use super::password_prompt_for_hostname;
    use std::os::unix::ffi::OsStrExt;

    #[test]
    fn formats_prompt_for_non_utf8_hostname_without_panicking() {
        let prompt =
            password_prompt_for_hostname("alice", std::ffi::OsStr::from_bytes(b"\xffhost"));

        assert!(prompt.starts_with("\rdoas (alice@"));
        assert!(prompt.ends_with(") password: "));
    }
}
