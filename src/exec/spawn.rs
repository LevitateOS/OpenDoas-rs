use std::{
    ffi::{CString, OsStr, OsString},
    fs,
    os::fd::RawFd,
    os::unix::ffi::{OsStrExt, OsStringExt},
    path::{Path, PathBuf},
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
    search_path: &OsStr,
    cmd_cstr: &CString,
    args: &[CString],
    env_cstrs: &[CString],
) -> Result<SpawnOutcome, String> {
    let child_pid = spawn_resolved(cmd, search_path, cmd_cstr, args, env_cstrs)?;

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

fn spawn_resolved(
    cmd: &str,
    search_path: &OsStr,
    cmd_cstr: &CString,
    args: &[CString],
    env_cstrs: &[CString],
) -> Result<nix::unistd::Pid, String> {
    if cmd.is_empty() {
        return Err(String::from(": command not found"));
    }

    let argv = build_argv(cmd_cstr, args);
    let mut long_path_warnings = Vec::new();

    if cmd.contains('/') {
        let path = Path::new(cmd);
        return match spawn_process(path, &argv, env_cstrs) {
            Ok(child_pid) => Ok(child_pid),
            Err(Errno::ENOEXEC) => spawn_shell_fallback(path, args, env_cstrs),
            Err(err) => Err(render_spawn_error(cmd, err)),
        };
    }

    let mut saw_eacces = false;

    for entry in path_entries(search_path) {
        let path = joined_candidate(&entry, cmd);
        match spawn_process(&path, &argv, env_cstrs) {
            Ok(child_pid) => {
                emit_path_search_warnings(&long_path_warnings);
                return Ok(child_pid);
            }
            Err(Errno::ENOEXEC) => {
                emit_path_search_warnings(&long_path_warnings);
                return spawn_shell_fallback(&path, args, env_cstrs);
            }
            Err(Errno::ENOENT | Errno::ENOTDIR) => continue,
            Err(Errno::EACCES) => saw_eacces = true,
            Err(Errno::ENAMETOOLONG) => long_path_warnings.push(entry),
            Err(err) => {
                emit_path_search_warnings(&long_path_warnings);
                return Err(render_spawn_error(cmd, err));
            }
        }
    }

    emit_path_search_warnings(&long_path_warnings);
    if saw_eacces {
        Err(format!("{cmd}: Permission denied"))
    } else {
        Err(format!("{cmd}: command not found"))
    }
}

fn build_argv(cmd_cstr: &CString, args: &[CString]) -> Vec<CString> {
    [cmd_cstr]
        .into_iter()
        .chain(args.iter())
        .cloned()
        .collect::<Vec<_>>()
}

fn spawn_process<P: ?Sized + nix::NixPath>(
    path: &P,
    argv: &[CString],
    env_cstrs: &[CString],
) -> Result<nix::unistd::Pid, Errno> {
    let mut file_actions = spawn::PosixSpawnFileActions::init().map_err(|_| Errno::EINVAL)?;
    for fd in inherited_fds().map_err(|_| Errno::EINVAL)? {
        file_actions.add_close(fd).map_err(|_| Errno::EINVAL)?;
    }

    spawn::posix_spawn(
        path,
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
    path: &Path,
    args: &[CString],
    env_cstrs: &[CString],
) -> Result<nix::unistd::Pid, String> {
    let shell_argv0 = CString::new("sh").map_err(|_| String::from("invalid shell argv"))?;
    let script_cstr = CString::new(path.as_os_str().as_bytes())
        .map_err(|_| String::from("invalid script path"))?;
    let argv = [shell_argv0, script_cstr]
        .into_iter()
        .chain(args.iter().cloned())
        .collect::<Vec<_>>();

    spawn_process(Path::new("/bin/sh"), &argv, env_cstrs)
        .map_err(|err| format!("posix_spawn: {err}"))
}

fn render_spawn_error(cmd: &str, err: Errno) -> String {
    match err {
        Errno::ENOENT => format!("{cmd}: command not found"),
        Errno::EACCES => format!("{cmd}: Permission denied"),
        Errno::ENAMETOOLONG => format!("execvp: {cmd}: path too long"),
        _ => format!("posix_spawn: {}", err.desc()),
    }
}

fn emit_path_search_warnings(entries: &[OsString]) {
    for entry in entries {
        eprintln!("execvp: {}: path too long", Path::new(entry).display());
    }
}

fn path_entries(search_path: &OsStr) -> Vec<OsString> {
    search_path
        .as_bytes()
        .split(|byte| *byte == b':')
        .map(|entry| {
            if entry.is_empty() {
                OsString::from(".")
            } else {
                OsString::from_vec(entry.to_vec())
            }
        })
        .collect()
}

fn joined_candidate(base: &OsStr, cmd: &str) -> PathBuf {
    PathBuf::from(base).join(cmd)
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
