use std::{collections::HashMap, env};

use pwd_grp::Passwd;

use crate::{
    exec::{
        env::build_exec_env,
        path::reset_process_path,
        privilege::switch_to_target,
        spawn::{spawn_and_wait, SpawnOutcome},
    },
    policy::command::get_cmdline,
    RuleOpts,
};

#[derive(Debug)]
pub struct ExecutionPlan<'a> {
    pub source: &'a Passwd,
    pub target: &'a Passwd,
    pub command: &'a str,
    pub args: &'a [String],
    pub rule_opts: &'a RuleOpts,
    pub source_env: &'a HashMap<String, String>,
    pub former_path: &'a str,
}

impl ExecutionPlan<'_> {
    pub fn cmdline(&self) -> String {
        get_cmdline(self.command, self.args)
    }
}

pub fn current_dir_label() -> String {
    match env::current_dir() {
        Ok(dir) => dir.to_str().unwrap_or("(invalid utf8)").to_string(),
        Err(_) => String::from("(failed)"),
    }
}

pub fn execute_plan(plan: &ExecutionPlan<'_>) -> Result<i32, String> {
    reset_process_path();

    let cmd_cstr = std::ffi::CString::new(plan.command)
        .map_err(|_| String::from("Invalid command"))?;
    let arg_cstrs: Vec<_> = plan
        .args
        .iter()
        .map(|arg| std::ffi::CString::new(arg.as_bytes()).map_err(|_| String::from("Invalid argument")))
        .collect::<Result<_, _>>()?;
    let env_cstrs = build_exec_env(
        plan.source,
        plan.target,
        plan.rule_opts,
        plan.source_env,
        plan.former_path,
    );

    switch_to_target(plan.target)?;
    reset_process_path();

    match spawn_and_wait(plan.command, &cmd_cstr, &arg_cstrs, &env_cstrs)? {
        SpawnOutcome::Exit(code) => Ok(code),
        SpawnOutcome::Signal(code) => Ok(code),
    }
}
