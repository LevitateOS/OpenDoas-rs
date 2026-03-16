use std::{borrow::Cow, ffi::CString};

use libc;

const SYSLOG_STRING_FORMAT: &[u8] = b"%s\0";

fn sanitize_syslog_message(msg: &str) -> Cow<'_, str> {
    if msg.as_bytes().contains(&0) {
        Cow::Owned(msg.replace('\0', "\\0"))
    } else {
        Cow::Borrowed(msg)
    }
}

fn syslog_message(priority: libc::c_int, msg: &str) {
    let sanitized = sanitize_syslog_message(msg);
    let Ok(message) = CString::new(sanitized.as_ref()) else {
        return;
    };

    // Pass a constant C format string so user-controlled text is treated as data.
    unsafe {
        libc::syslog(
            priority,
            SYSLOG_STRING_FORMAT.as_ptr().cast(),
            message.as_ptr(),
        );
    }
}

pub fn log_denied_command(user: &str, cmdline: &str) {
    let msg = format!("command not permitted for {}: {}", user, cmdline);
    syslog_message(libc::LOG_AUTHPRIV | libc::LOG_NOTICE, &msg);
}

pub fn log_failed_auth(user: &str) {
    let msg = format!("failed auth for {}", user);
    syslog_message(libc::LOG_AUTHPRIV | libc::LOG_NOTICE, &msg);
}

pub fn log_tty_required(user: &str) {
    let msg = format!("tty required for {}", user);
    syslog_message(libc::LOG_AUTHPRIV | libc::LOG_NOTICE, &msg);
}

pub fn log_permitted_command(user: &str, cmdline: &str, target: &str, cwd: &str) {
    let msg = format!(
        "{} ran command {} as {} from {}",
        user, cmdline, target, cwd
    );
    syslog_message(libc::LOG_AUTHPRIV | libc::LOG_INFO, &msg);
}

#[cfg(test)]
mod tests {
    use super::sanitize_syslog_message;

    #[test]
    fn preserves_percent_sequences_as_data() {
        assert_eq!(sanitize_syslog_message("%n %s %%").as_ref(), "%n %s %%");
    }

    #[test]
    fn replaces_nul_bytes_before_cstring_conversion() {
        assert_eq!(sanitize_syslog_message("abc\0def").as_ref(), "abc\\0def");
    }
}
