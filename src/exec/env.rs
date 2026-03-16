use std::{
    collections::BTreeMap,
    ffi::{CString, OsStr, OsString},
    os::unix::ffi::OsStrExt,
};

use pwd_grp::Passwd;

use crate::policy::rule::{EnvDirective, RuleOpts};

use super::path::SAFE_PATH;

pub type SourceEnv = BTreeMap<OsString, OsString>;

pub fn collect_source_env(vars: impl IntoIterator<Item = (OsString, OsString)>) -> SourceEnv {
    let mut source_env = SourceEnv::new();
    for (key, value) in vars {
        source_env.entry(key).or_insert(value);
    }
    source_env
}

pub fn env_cstr(key: &OsStr, value: &OsStr) -> Result<CString, String> {
    let mut env_bytes = Vec::with_capacity(key.as_bytes().len() + value.as_bytes().len() + 1);
    env_bytes.extend_from_slice(key.as_bytes());
    env_bytes.push(b'=');
    env_bytes.extend_from_slice(value.as_bytes());

    CString::new(env_bytes).map_err(|_| String::from("Invalid environment entry"))
}

pub fn build_exec_env(
    passwd: &Passwd,
    passwd_target: &Passwd,
    rule_opts: &RuleOpts,
    source_env: &SourceEnv,
    former_path: &OsStr,
) -> Result<Vec<CString>, String> {
    let mut env_map = BTreeMap::new();

    env_map.insert(OsString::from("DOAS_USER"), OsString::from(&passwd.name));
    env_map.insert(OsString::from("HOME"), OsString::from(&passwd_target.dir));
    env_map.insert(
        OsString::from("LOGNAME"),
        OsString::from(&passwd_target.name),
    );
    env_map.insert(OsString::from("PATH"), OsString::from(SAFE_PATH));
    env_map.insert(
        OsString::from("SHELL"),
        OsString::from(&passwd_target.shell),
    );
    env_map.insert(OsString::from("USER"), OsString::from(&passwd_target.name));

    for key in ["DISPLAY", "TERM"] {
        if let Some(value) = source_env.get(OsStr::new(key)) {
            env_map.insert(OsString::from(key), value.clone());
        }
    }

    if rule_opts.keepenv {
        for (key, value) in source_env {
            env_map.entry(key.clone()).or_insert_with(|| value.clone());
        }
    }

    if let Some(directives) = &rule_opts.setenv {
        apply_setenv_directives(&mut env_map, directives, source_env, former_path);
    }

    env_map
        .into_iter()
        .map(|(key, value)| env_cstr(&key, &value))
        .collect()
}

fn apply_setenv_directives(
    env_map: &mut BTreeMap<OsString, OsString>,
    directives: &[EnvDirective],
    source_env: &SourceEnv,
    former_path: &OsStr,
) {
    for directive in directives {
        match directive {
            EnvDirective::Remove(name) => {
                env_map.remove(OsStr::new(name));
            }
            EnvDirective::Inherit(name) => {
                env_map.remove(OsStr::new(name));
                if let Some(value) = inherited_value(name, source_env, former_path) {
                    env_map.insert(OsString::from(name), value);
                }
            }
            EnvDirective::Set(name, value) => {
                env_map.remove(OsStr::new(name));
                if let Some(expanded) = assigned_value(value, source_env, former_path) {
                    env_map.insert(OsString::from(name), expanded);
                }
            }
        }
    }
}

fn inherited_value(name: &str, source_env: &SourceEnv, former_path: &OsStr) -> Option<OsString> {
    if name == "PATH" {
        Some(former_path.to_os_string())
    } else {
        source_env.get(OsStr::new(name)).cloned()
    }
}

fn assigned_value(value: &str, source_env: &SourceEnv, former_path: &OsStr) -> Option<OsString> {
    if let Some(name) = value.strip_prefix('$') {
        inherited_value(name, source_env, former_path)
    } else {
        Some(OsString::from(value))
    }
}

#[cfg(test)]
mod tests {
    use super::{build_exec_env, collect_source_env, env_cstr, SourceEnv};
    use crate::RuleOpts;
    use pwd_grp::Passwd;
    use std::{
        ffi::{OsStr, OsString},
        os::unix::ffi::OsStringExt,
    };

    fn sample_passwd(name: &str, dir: &str, shell: &str) -> Passwd {
        Passwd {
            name: String::from(name),
            passwd: String::from("x"),
            uid: 1000,
            gid: 1000,
            gecos: String::new(),
            dir: String::from(dir),
            shell: String::from(shell),
            __non_exhaustive: Default::default(),
        }
    }

    fn default_rule_opts() -> RuleOpts {
        RuleOpts {
            nopass: false,
            nolog: false,
            persist: false,
            keepenv: false,
            setenv: None,
        }
    }

    #[test]
    fn env_cstr_preserves_non_utf8_bytes() {
        let key = OsString::from("DISPLAY");
        let value = OsString::from_vec(vec![0xff, b':', b'9', b'9']);
        let entry = env_cstr(&key, &value).expect("expected valid env cstring");

        assert_eq!(entry.as_c_str().to_bytes(), b"DISPLAY=\xff:99");
    }

    #[test]
    fn build_exec_env_keeps_non_utf8_source_values() {
        let source = sample_passwd("alice", "/home/alice", "/bin/sh");
        let target = sample_passwd("root", "/root", "/bin/sh");
        let mut source_env = SourceEnv::new();
        source_env.insert(
            OsString::from("DISPLAY"),
            OsString::from_vec(vec![0xff, b':', b'1']),
        );

        let env = build_exec_env(
            &source,
            &target,
            &default_rule_opts(),
            &source_env,
            OsString::from("/usr/bin").as_os_str(),
        )
        .expect("expected env build to succeed");

        assert!(env
            .iter()
            .any(|entry| entry.as_c_str().to_bytes() == b"DISPLAY=\xff:1"));
    }

    #[test]
    fn collect_source_env_keeps_first_duplicate_value() {
        let source_env = collect_source_env([
            (OsString::from("DISPLAY"), OsString::from(":1")),
            (OsString::from("DISPLAY"), OsString::from(":2")),
        ]);

        assert_eq!(
            source_env.get(OsStr::new("DISPLAY")),
            Some(&OsString::from(":1"))
        );
    }
}
