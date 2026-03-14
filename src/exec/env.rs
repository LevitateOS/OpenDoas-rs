use std::{collections::BTreeMap, collections::HashMap, ffi::CString};

use pwd_grp::Passwd;

use crate::policy::rule::{EnvDirective, RuleOpts};

use super::path::SAFE_PATH;

pub fn env_cstr(key: &str, value: &str) -> CString {
    let mut env_str = String::from(key);
    env_str.push('=');
    env_str.push_str(value);

    unsafe { CString::new(env_str).unwrap_unchecked() }
}

pub fn build_exec_env(
    passwd: &Passwd,
    passwd_target: &Passwd,
    rule_opts: &RuleOpts,
    source_env: &HashMap<String, String>,
    former_path: &str,
) -> Vec<CString> {
    let mut env_map = BTreeMap::new();

    env_map.insert(String::from("DOAS_USER"), passwd.name.clone());
    env_map.insert(String::from("HOME"), passwd_target.dir.clone());
    env_map.insert(String::from("LOGNAME"), passwd_target.name.clone());
    env_map.insert(String::from("PATH"), String::from(SAFE_PATH));
    env_map.insert(String::from("SHELL"), passwd_target.shell.clone());
    env_map.insert(String::from("USER"), passwd_target.name.clone());

    for key in ["DISPLAY", "TERM"] {
        if let Some(value) = source_env.get(key) {
            env_map.insert(key.to_string(), value.clone());
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
    env_map: &mut BTreeMap<String, String>,
    directives: &[EnvDirective],
    source_env: &HashMap<String, String>,
    former_path: &str,
) {
    for directive in directives {
        match directive {
            EnvDirective::Remove(name) => {
                env_map.remove(name);
            }
            EnvDirective::Inherit(name) => {
                env_map.remove(name);
                if let Some(value) = inherited_value(name, source_env, former_path) {
                    env_map.insert(name.clone(), value);
                }
            }
            EnvDirective::Set(name, value) => {
                env_map.remove(name);
                if let Some(expanded) = assigned_value(value, source_env, former_path) {
                    env_map.insert(name.clone(), expanded);
                }
            }
        }
    }
}

fn inherited_value(
    name: &str,
    source_env: &HashMap<String, String>,
    former_path: &str,
) -> Option<String> {
    if name == "PATH" {
        Some(former_path.to_string())
    } else {
        source_env.get(name).cloned()
    }
}

fn assigned_value(
    value: &str,
    source_env: &HashMap<String, String>,
    former_path: &str,
) -> Option<String> {
    if let Some(name) = value.strip_prefix('$') {
        inherited_value(name, source_env, former_path)
    } else {
        Some(value.to_string())
    }
}
