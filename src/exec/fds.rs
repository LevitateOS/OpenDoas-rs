use std::{fs, os::fd::RawFd};

pub fn inherited_fds_from(min_fd: RawFd) -> Result<Vec<RawFd>, std::io::Error> {
    let mut fds = Vec::new();
    for entry in fs::read_dir("/proc/self/fd")? {
        let entry = entry?;
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        let Ok(fd) = name.parse::<RawFd>() else {
            continue;
        };
        if fd > min_fd {
            fds.push(fd);
        }
    }
    fds.sort_unstable();
    fds.dedup();
    Ok(fds)
}

pub fn close_fds(fds: impl IntoIterator<Item = RawFd>) -> Result<(), String> {
    for fd in fds {
        let rc = unsafe { libc::close(fd) };
        if rc == 0 {
            continue;
        }

        let err = std::io::Error::last_os_error();
        if err.raw_os_error() == Some(libc::EBADF) {
            continue;
        }
        return Err(format!("close({fd}): {err}"));
    }

    Ok(())
}

pub fn close_inherited_fds_from(min_fd: RawFd) -> Result<(), String> {
    let fds = inherited_fds_from(min_fd).map_err(|err| err.to_string())?;
    close_fds(fds)
}

#[cfg(test)]
mod tests {
    use super::{close_fds, inherited_fds_from};
    use std::os::fd::{AsRawFd, IntoRawFd};

    #[test]
    fn inherited_fds_lists_open_fd() {
        let file = std::fs::File::open("/dev/null").expect("expected /dev/null to open");
        let fd = file.as_raw_fd();

        let fds = inherited_fds_from(libc::STDERR_FILENO).expect("expected fd scan to succeed");

        assert!(fds.contains(&fd));
    }

    #[test]
    fn close_fds_closes_owned_fd() {
        let file = std::fs::File::open("/dev/null").expect("expected /dev/null to open");
        let fd = file.into_raw_fd();

        close_fds([fd]).expect("expected close to succeed");

        let rc = unsafe { libc::fcntl(fd, libc::F_GETFD) };
        assert_eq!(rc, -1);
        assert_eq!(
            std::io::Error::last_os_error().raw_os_error(),
            Some(libc::EBADF)
        );
    }
}
