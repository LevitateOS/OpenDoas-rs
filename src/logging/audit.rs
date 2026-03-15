use libc;
use syslog_c::syslog;

pub fn log_denied_command(user: &str, cmdline: &str) {
    let msg = format!("command not permitted for {}: {}", user, cmdline);
    syslog(libc::LOG_AUTHPRIV | libc::LOG_NOTICE, &msg);
}

pub fn log_failed_auth(user: &str) {
    let msg = format!("failed auth for {}", user);
    syslog(libc::LOG_AUTHPRIV | libc::LOG_NOTICE, &msg);
}

pub fn log_tty_required(user: &str) {
    let msg = format!("tty required for {}", user);
    syslog(libc::LOG_AUTHPRIV | libc::LOG_NOTICE, &msg);
}

pub fn log_permitted_command(user: &str, cmdline: &str, target: &str, cwd: &str) {
    let msg = format!(
        "{} was permitted to run command {} as {} from {}",
        user, cmdline, target, cwd
    );
    syslog(libc::LOG_AUTHPRIV | libc::LOG_INFO, &msg);
}
