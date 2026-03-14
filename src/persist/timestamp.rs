use std::{
    fs::{self, File, OpenOptions},
    io::{ErrorKind, Read},
    os::{
        fd::AsRawFd,
        unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
    },
    path::{Path, PathBuf},
};

use nix::unistd::{getgid, getppid, getsid, getuid};

const TIMESTAMP_DIR: &str = "/run/doas";
const TIMESTAMP_TIMEOUT_SECS: i64 = 5 * 60;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PersistMode {
    Disabled,
    Enabled,
}

pub struct TimestampHandle {
    file: File,
    valid: bool,
}

impl TimestampHandle {
    pub fn is_valid(&self) -> bool {
        self.valid
    }

    pub fn refresh(&self) -> Result<(), String> {
        set_timestamp_deadline(&self.file, TIMESTAMP_TIMEOUT_SECS)
    }
}

impl PersistMode {
    pub fn from_enabled(enabled: bool) -> Self {
        if enabled {
            Self::Enabled
        } else {
            Self::Disabled
        }
    }

    pub fn is_enabled(self) -> bool {
        matches!(self, Self::Enabled)
    }
}

pub fn configured_persist_mode() -> PersistMode {
    let enabled = matches!(env!("OPENDOAS_RS_TIMESTAMP_MODE"), "on");
    PersistMode::from_enabled(enabled)
}

pub fn open_timestamp() -> Result<Option<TimestampHandle>, String> {
    if !configured_persist_mode().is_enabled() {
        return Ok(None);
    }

    if ensure_timestamp_dir().is_err() {
        return Ok(None);
    }
    let Ok(path) = current_timestamp_path() else {
        return Ok(None);
    };

    match open_existing_timestamp(&path)? {
        Some(file) => {
            let valid = validate_timestamp(&file, TIMESTAMP_TIMEOUT_SECS)?;
            Ok(Some(TimestampHandle { file, valid }))
        }
        None => match create_timestamp_file(&path) {
            Ok(file) => Ok(Some(TimestampHandle { file, valid: false })),
            Err(_) => Ok(None),
        },
    }
}

pub fn timestamp_clear() -> Result<(), String> {
    if !configured_persist_mode().is_enabled() {
        return Ok(());
    }

    let path = current_timestamp_path()?;
    match fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err.to_string()),
    }
}

fn ensure_timestamp_dir() -> Result<(), String> {
    match fs::symlink_metadata(TIMESTAMP_DIR) {
        Ok(metadata) => {
            let mode = metadata.permissions().mode() & 0o7777;
            if !metadata.is_dir() || metadata.uid() != 0 || mode != 0o700 {
                return Err(String::from("failed to open timestamp directory"));
            }
            Ok(())
        }
        Err(err) if err.kind() == ErrorKind::NotFound => {
            fs::create_dir(TIMESTAMP_DIR).map_err(|err| err.to_string())?;
            fs::set_permissions(TIMESTAMP_DIR, fs::Permissions::from_mode(0o700))
                .map_err(|err| err.to_string())
        }
        Err(err) => Err(err.to_string()),
    }
}

fn current_timestamp_path() -> Result<PathBuf, String> {
    let ppid = getppid().as_raw();
    let sid = getsid(None).map_err(|err| err.to_string())?.as_raw();
    let (ttynr, starttime) = parent_proc_info(ppid)?;
    Ok(PathBuf::from(format!(
        "{TIMESTAMP_DIR}/{ppid}-{sid}-{ttynr}-{starttime}-{}",
        getuid().as_raw()
    )))
}

fn parent_proc_info(pid: libc::pid_t) -> Result<(i32, u64), String> {
    let mut stat = String::new();
    File::open(format!("/proc/{pid}/stat"))
        .and_then(|mut file| file.read_to_string(&mut stat))
        .map_err(|err| err.to_string())?;
    let Some(end) = stat.rfind(')') else {
        return Err(String::from("failed to parse /proc stat"));
    };
    let fields: Vec<_> = stat[end + 1..].split_whitespace().collect();
    let ttynr = fields
        .get(4)
        .ok_or_else(|| String::from("failed to parse ttynr"))?
        .parse::<i32>()
        .map_err(|err| err.to_string())?;
    let starttime = fields
        .get(19)
        .ok_or_else(|| String::from("failed to parse starttime"))?
        .parse::<u64>()
        .map_err(|err| err.to_string())?;
    Ok((ttynr, starttime))
}

fn open_existing_timestamp(path: &Path) -> Result<Option<File>, String> {
    let mut options = OpenOptions::new();
    options
        .read(true)
        .write(true)
        .custom_flags(libc::O_NOFOLLOW);
    match options.open(path) {
        Ok(file) => Ok(Some(file)),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(None),
        Err(err) => Err(format!("open: {}: {}", path.display(), err)),
    }
}

fn create_timestamp_file(path: &Path) -> Result<File, String> {
    let tmp_path = PathBuf::from(format!("{TIMESTAMP_DIR}/.tmp-{}", std::process::id()));
    let mut options = OpenOptions::new();
    options
        .read(true)
        .write(true)
        .create_new(true)
        .mode(0o000)
        .custom_flags(libc::O_NOFOLLOW);

    let file = options.open(&tmp_path).map_err(|err| err.to_string())?;
    unsafe {
        if libc::fchown(file.as_raw_fd(), 0, getgid().as_raw()) != 0 {
            let err = std::io::Error::last_os_error().to_string();
            let _ = fs::remove_file(&tmp_path);
            return Err(err);
        }
    }
    clear_file_times(&file)?;
    fs::rename(&tmp_path, path).map_err(|err| err.to_string())?;
    Ok(file)
}

fn validate_timestamp(file: &File, secs: i64) -> Result<bool, String> {
    let mut st = std::mem::MaybeUninit::<libc::stat>::uninit();
    let rc = unsafe { libc::fstat(file.as_raw_fd(), st.as_mut_ptr()) };
    if rc != 0 {
        return Err(std::io::Error::last_os_error().to_string());
    }
    let st = unsafe { st.assume_init() };
    let mode = st.st_mode & 0o7777;
    let file_type = st.st_mode & libc::S_IFMT;
    if st.st_uid != 0 || st.st_gid != getgid().as_raw() || file_type != libc::S_IFREG || mode != 0 {
        return Err(String::from("timestamp uid, gid or mode wrong"));
    }

    let atime = stat_atime(&st);
    let mtime = stat_mtime(&st);
    if !timespec_is_set(atime) || !timespec_is_set(mtime) {
        return Ok(false);
    }

    let now = current_times()?;
    if timespec_cmp(atime, now[0]) < 0 || timespec_cmp(mtime, now[1]) < 0 {
        return Ok(false);
    }

    let max = [
        timespec_add_seconds(now[0], secs),
        timespec_add_seconds(now[1], secs),
    ];
    if timespec_cmp(atime, max[0]) > 0 || timespec_cmp(mtime, max[1]) > 0 {
        return Ok(false);
    }

    Ok(true)
}

fn set_timestamp_deadline(file: &File, secs: i64) -> Result<(), String> {
    let now = current_times()?;
    let deadline = [
        timespec_add_seconds(now[0], secs),
        timespec_add_seconds(now[1], secs),
    ];
    let rc = unsafe { libc::futimens(file.as_raw_fd(), deadline.as_ptr()) };
    if rc == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error().to_string())
    }
}

fn clear_file_times(file: &File) -> Result<(), String> {
    let zero = [
        libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        },
        libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        },
    ];
    let rc = unsafe { libc::futimens(file.as_raw_fd(), zero.as_ptr()) };
    if rc == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error().to_string())
    }
}

fn current_times() -> Result<[libc::timespec; 2], String> {
    let mut boot = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    let mut real = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };

    let boot_ok = unsafe { libc::clock_gettime(libc::CLOCK_BOOTTIME, &mut boot) };
    if boot_ok != 0 {
        return Err(std::io::Error::last_os_error().to_string());
    }
    let real_ok = unsafe { libc::clock_gettime(libc::CLOCK_REALTIME, &mut real) };
    if real_ok != 0 {
        return Err(std::io::Error::last_os_error().to_string());
    }

    Ok([boot, real])
}

fn timespec_is_set(ts: libc::timespec) -> bool {
    ts.tv_sec != 0 || ts.tv_nsec != 0
}

fn stat_atime(st: &libc::stat) -> libc::timespec {
    libc::timespec {
        tv_sec: st.st_atime,
        tv_nsec: st.st_atime_nsec,
    }
}

fn stat_mtime(st: &libc::stat) -> libc::timespec {
    libc::timespec {
        tv_sec: st.st_mtime,
        tv_nsec: st.st_mtime_nsec,
    }
}

fn timespec_add_seconds(mut ts: libc::timespec, secs: i64) -> libc::timespec {
    ts.tv_sec += secs;
    ts
}

fn timespec_cmp(left: libc::timespec, right: libc::timespec) -> i32 {
    match left.tv_sec.cmp(&right.tv_sec) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Greater => 1,
        std::cmp::Ordering::Equal => match left.tv_nsec.cmp(&right.tv_nsec) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Greater => 1,
            std::cmp::Ordering::Equal => 0,
        },
    }
}
