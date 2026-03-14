/*
OpenDoas-rs - Privilege escalation utility
Copyright (C) 2023  TheDcoder <TheDcoder@protonmail.com>

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

#[allow(unused_imports)]
use open_doas_rs::{
    app::{load_rules, render_check_result, ConfigRequest},
    auth::*,
    command::*,
    exec::{
        privilege::{drop_to_real_uid, ensure_setuid_root},
        run::{current_dir_label, execute_plan, ExecutionPlan},
        shell::selected_command,
    },
    logging::{log_denied_command, log_failed_auth, log_permitted_command},
    persist,
    platform::{current_group_info, current_passwd, target_passwd},
    policy::{command::get_cmdline, Decision},
};
use std::{collections::HashMap, env};

fn main() {
    match Command::new() {
        Command::Execute(opts) => execute(opts),
        Command::Deauth => {
            if let Err(err) = persist::deauth() {
                print_error_and_exit(&err, 1);
            }
        }
    };
}

fn execute(opts: Execute) {
    let config_request = ConfigRequest::from_execute(&opts);
    if config_request.only_check {
        if let Err(err) = drop_to_real_uid() {
            print_error_and_exit(&err, 1);
        }
    } else if let Err(err) = ensure_setuid_root() {
        print_error_and_exit(&err, 1);
    }

    let rules = load_rules(&config_request).unwrap_or_else(|err| print_error_and_exit(&err, 1));

    let passwd = current_passwd().unwrap_or_else(|err| print_error_and_exit(&err, 1));
    let source_env: HashMap<String, String> = env::vars().collect();
    let former_path = source_env.get("PATH").cloned().unwrap_or_default();
    let (groups, gids) =
        current_group_info(passwd.gid.into()).unwrap_or_else(|err| print_error_and_exit(&err, 1));
    if config_request.only_check {
        if let Some(cmd) = opts.cmd.as_deref() {
            let decision = rules.decide(
                &passwd.name,
                passwd.uid.into(),
                &groups,
                &gids,
                cmd,
                &opts.args,
                opts.target_uid,
            );
            let (output, exit_code) = render_check_result(decision);
            print!("{output}");
            std::process::exit(exit_code);
        }
        return;
    }

    let passwd_target =
        target_passwd(opts.target_uid).unwrap_or_else(|err| print_error_and_exit(&err, 1));

    let cmd = selected_command(opts.cmd.clone(), &passwd);
    let decision = rules.decide(
        &passwd.name,
        passwd.uid.into(),
        &groups,
        &gids,
        &cmd,
        &opts.args,
        opts.target_uid,
    );

    let rule_opts = match decision {
        Decision::Deny => {
            let cmdline = get_cmdline(&cmd, &opts.args);
            log_denied_command(&passwd.name, &cmdline);
            print_error_and_exit("Not permitted", 1);
        }
        Decision::Permit(match_opts) => match_opts,
    };

    let plan = ExecutionPlan {
        source: &passwd,
        target: &passwd_target,
        command: &cmd,
        args: &opts.args,
        rule_opts: &rule_opts,
        source_env: &source_env,
        former_path: &former_path,
    };

    let timestamp = if rule_opts.persist {
        persist::open_timestamp().unwrap_or_else(|err| print_error_and_exit(&err, 1))
    } else {
        None
    };
    #[allow(unused_variables)]
    let reuse_persist = timestamp
        .as_ref()
        .map(|handle| handle.is_valid())
        .unwrap_or(false);
    let run_result;

    #[cfg(auth = "none")]
    {
        if let Err(msg) = ensure_nopass(&rule_opts) {
            print_error_and_exit(msg, 1);
        }

        run_result = run_permitted_command(&plan);
    }

    #[cfg(auth = "pam")]
    {
        use pam_client;

        let mut pam_context;
        let mut pam_session = None;
        let require_auth = !rule_opts.nopass && !reuse_persist;

        if require_auth && !opts.interactive {
            print_error_and_exit("Authentication required", 1);
        }
        match authenticate(&passwd, &passwd_target, require_auth) {
            Ok(transaction) => {
                pam_context = transaction.into_context();
                pam_session = Some(
                    pam_context
                        .open_session(pam_client::Flag::NONE)
                        .unwrap_or_else(|err| {
                            print_error_and_exit(&format!("Failed to start PAM session: {err}"), 1)
                        }),
                );
            }
            Err(msg) => {
                if msg == "Authentication failed" {
                    log_failed_auth(&passwd.name);
                }
                print_error_and_exit(msg, 1);
            }
        }

        if let Some(handle) = timestamp.as_ref() {
            handle
                .refresh()
                .unwrap_or_else(|err| print_error_and_exit(&err, 1));
        }
        run_result = run_permitted_command_with_privileged_parent(&plan);

        if let Some(session) = pam_session {
            if let Err(err) = session.close(pam_client::Flag::NONE) {
                print_error_and_exit(&format!("Failed to close PAM session: {err}"), 1);
            }
        }
    }

    #[cfg(auth = "plain")]
    {
        if !rule_opts.nopass && !reuse_persist {
            if !opts.interactive {
                print_error_and_exit("Authentication required", 1);
            }
            if let Err(msg) = challenge_user(&passwd) {
                if msg == "Authentication failed" {
                    log_failed_auth(&passwd.name);
                }
                print_error_and_exit(msg, 1);
            }
        }

        if let Some(handle) = timestamp.as_ref() {
            handle
                .refresh()
                .unwrap_or_else(|err| print_error_and_exit(&err, 1));
        }
        run_result = run_permitted_command(&plan);
    }

    match run_result {
        Ok(code) => {
            if code != 0 {
                std::process::exit(code);
            }
        }
        Err(msg) => print_error_and_exit(&format!("Error while trying to run: {msg}"), 1),
    }
}

fn run_permitted_command(plan: &ExecutionPlan<'_>) -> Result<i32, String> {
    if !plan.rule_opts.nolog {
        let cwd = current_dir_label();
        log_permitted_command(&plan.source.name, &plan.cmdline(), &plan.target.name, &cwd);
    }

    execute_plan(plan)
}

#[cfg(auth = "pam")]
fn authenticate<'a>(
    source: &'a pwd_grp::Passwd,
    target: &'a pwd_grp::Passwd,
    require_auth: bool,
) -> Result<Transaction<'a>, &'static str> {
    let mut transaction = Transaction::new();
    transaction.begin(source, target, require_auth)?;
    Ok(transaction)
}

#[cfg(auth = "pam")]
fn run_permitted_command_with_privileged_parent(plan: &ExecutionPlan<'_>) -> Result<i32, String> {
    use nix::{
        sys::wait::{waitpid, WaitStatus},
        unistd::{fork, ForkResult},
    };

    match unsafe { fork() }.map_err(|err| format!("fork: {err}"))? {
        ForkResult::Child => {
            let code = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                run_permitted_command(plan)
            })) {
                Ok(Ok(code)) => code,
                Ok(Err(msg)) => {
                    print_error(&format!("Error while trying to run: {msg}"));
                    1
                }
                Err(_) => {
                    print_error("Error while trying to run: panic during command execution");
                    1
                }
            };
            std::process::exit(code);
        }
        ForkResult::Parent { child } => loop {
            match waitpid(Some(child), None) {
                Ok(WaitStatus::Exited(_, code)) => return Ok(code),
                Ok(WaitStatus::Signaled(_, signal, _)) => return Ok(128 + signal as i32),
                Ok(_) => (),
                Err(errno) => return Err(format!("waitpid: {}", errno.desc())),
            }
        },
    }
}
