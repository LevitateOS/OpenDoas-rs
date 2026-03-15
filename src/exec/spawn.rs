use std::{
    env,
    ffi::CString,
    fs,
    os::{fd::RawFd, unix::fs::PermissionsExt},
    path::PathBuf,
};

use nix::{
    errno::Errno,
    spawn,
    sys::wait::{waitpid, WaitStatus},
};

#[derive(Debug)]
pub enum SpawnOutcome {
    Exit(i32),
    Signal(i32),
}

pub fn spawn_and_wait(
    cmd: &str,
    cmd_cstr: &CString,
    args: &[CString],
    env_cstrs: &[CString],
) -> Result<SpawnOutcome, String> {
    let child_pid = match spawn_process(
        cmd_cstr,
        [cmd_cstr]
            .into_iter()
            .chain(args.iter())
            .cloned()
            .collect::<Vec<_>>(),
        env_cstrs,
    ) {
        Ok(child_pid) => child_pid,
        Err(Errno::ENOEXEC) => spawn_shell_fallback(cmd, args, env_cstrs)?,
        Err(err) => {
            return Err(match err {
                Errno::ENOENT => format!("{cmd}: command not found"),
                Errno::EACCES => format!("{cmd}: Permission denied"),
                Errno::ENAMETOOLONG => format!("execvp: {cmd}: path too long"),
                _ => format!("posix_spawnp: {}", err.desc()),
            })
        }
    };

    loop {
        match waitpid(Some(child_pid), None) {
            Ok(WaitStatus::Exited(_, code)) => return Ok(SpawnOutcome::Exit(code)),
            Ok(WaitStatus::Signaled(_, signal, _)) => {
                return Ok(SpawnOutcome::Signal(128 + signal as i32))
            }
            Ok(_) => (),
            Err(errno) => {
                if errno == Errno::ENOENT {
                    return Err(format!("{}: command not found", cmd));
                }
                return Err(format!("waitpid: {}", errno.desc()));
            }
        }
    }
}

fn spawn_process(
    cmd_cstr: &CString,
    argv: Vec<CString>,
    env_cstrs: &[CString],
) -> Result<nix::unistd::Pid, Errno> {
    let mut file_actions = spawn::PosixSpawnFileActions::init().map_err(|_| Errno::EINVAL)?;
    for fd in inherited_fds().map_err(|_| Errno::EINVAL)? {
        file_actions.add_close(fd).map_err(|_| Errno::EINVAL)?;
    }

    spawn::posix_spawnp(
        cmd_cstr.as_c_str(),
        &file_actions,
        &spawn::PosixSpawnAttr::init().map_err(|_| Errno::EINVAL)?,
        &argv
            .iter()
            .map(|value| value.as_c_str())
            .collect::<Vec<_>>(),
        &env_cstrs
            .iter()
            .map(|value| value.as_c_str())
            .collect::<Vec<_>>(),
    )
}

fn spawn_shell_fallback(
    cmd: &str,
    args: &[CString],
    env_cstrs: &[CString],
) -> Result<nix::unistd::Pid, String> {
    let script_path =
        resolve_shell_fallback_path(cmd).ok_or_else(|| format!("{cmd}: command not found"))?;
    let shell_cstr = CString::new("/bin/sh").map_err(|_| String::from("invalid shell path"))?;
    let shell_argv0 = CString::new("sh").map_err(|_| String::from("invalid shell argv"))?;
    let script_cstr = CString::new(script_path).map_err(|_| String::from("invalid script path"))?;
    let argv = [shell_argv0, script_cstr]
        .into_iter()
        .chain(args.iter().cloned())
        .collect::<Vec<_>>();

    spawn_process(&shell_cstr, argv, env_cstrs).map_err(|err| format!("posix_spawnp: {err}"))
}

fn resolve_shell_fallback_path(cmd: &str) -> Option<String> {
    if cmd.contains('/') {
        return Some(cmd.to_string());
    }

    env::var("PATH").ok().and_then(|path| {
        path.split(':').find_map(|entry| {
            let base = if entry.is_empty() { "." } else { entry };
            let candidate = PathBuf::from(base).join(cmd);
            let metadata = candidate.metadata().ok()?;
            if !metadata.is_file() || metadata.permissions().mode() & 0o111 == 0 {
                return None;
            }
            Some(candidate.to_string_lossy().into_owned())
        })
    })
}

fn inherited_fds() -> Result<Vec<RawFd>, std::io::Error> {
    let mut fds = Vec::new();
    for entry in fs::read_dir("/proc/self/fd")? {
        let entry = entry?;
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        let Ok(fd) = name.parse::<RawFd>() else {
            continue;
        };
        if fd > libc::STDERR_FILENO {
            fds.push(fd);
        }
    }
    fds.sort_unstable();
    fds.dedup();
    Ok(fds)
}
