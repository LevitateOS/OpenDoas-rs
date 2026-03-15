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
        spawn::SpawnOutcome,
    },
    logging::{log_denied_command, log_failed_auth, log_permitted_command, log_tty_required},
    persist,
    platform::{current_group_info, current_passwd, target_passwd},
    policy::{command::get_cmdline, Decision},
};
use std::ffi::CStr;
#[cfg(auth = "pam")]
use std::sync::atomic::{AtomicI32, Ordering};
use std::{collections::HashMap, env};

#[cfg(auth = "pam")]
static CAUGHT_SESSION_SIGNAL: AtomicI32 = AtomicI32::new(0);

#[cfg(auth = "pam")]
extern "C" fn catch_session_signal(signal: libc::c_int) {
    CAUGHT_SESSION_SIGNAL.store(signal, Ordering::Relaxed);
}

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

    let passwd = current_passwd().unwrap_or_else(|err| print_error_and_exit(&err, 1));
    let rules = load_rules(&config_request).unwrap_or_else(|err| print_error_and_exit(&err, 1));
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
            print_error_and_exit("Operation not permitted", 1);
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

        let require_auth = !rule_opts.nopass && !reuse_persist;

        if require_auth && !opts.interactive {
            print_error_and_exit("Authentication required", 1);
        }
        let transaction = match authenticate(&passwd, &passwd_target, require_auth) {
            Ok(transaction) => transaction,
            Err(msg) => {
                log_auth_failure(&passwd.name, msg);
                print_error_and_exit(msg, 1);
            }
        };
        run_result = {
            let mut pam_context = transaction.into_context();
            let pam_session = pam_context
                .open_session(pam_client::Flag::NONE)
                .unwrap_or_else(|err| print_error_and_exit(&format!("pam_open_session: {err}"), 1));

            if let Some(handle) = timestamp.as_ref() {
                handle
                    .refresh()
                    .unwrap_or_else(|err| print_error_and_exit(&err, 1));
            }
            run_permitted_command_with_privileged_parent(&plan, pam_session)
        };
    }

    #[cfg(auth = "plain")]
    {
        if !rule_opts.nopass && !reuse_persist {
            if !opts.interactive {
                print_error_and_exit("Authentication required", 1);
            }
            if let Err(msg) = challenge_user(&passwd) {
                log_auth_failure(&passwd.name, msg);
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
        Ok(SpawnOutcome::Exit(code)) => {
            if code != 0 {
                std::process::exit(code);
            }
        }
        Ok(SpawnOutcome::Signal(code)) => {
            print_signal_message(code - 128, false);
            std::process::exit(code);
        }
        Err(msg) => print_exec_error_and_exit(&msg, 1),
    }
}

fn print_exec_error_and_exit(msg: &str, code: i32) -> ! {
    if msg.starts_with("execvp: ") {
        eprintln!("{msg}");
        std::process::exit(code);
    }
    print_error_and_exit(msg, code)
}

fn run_permitted_command(plan: &ExecutionPlan<'_>) -> Result<SpawnOutcome, String> {
    if !plan.rule_opts.nolog {
        let cwd = current_dir_label();
        log_permitted_command(&plan.source.name, &plan.cmdline(), &plan.target.name, &cwd);
    }

    execute_plan(plan)
}

fn log_auth_failure(user: &str, msg: &str) {
    match msg {
        "Authentication failed" => log_failed_auth(user),
        "a tty is required" => log_tty_required(user),
        _ => (),
    }
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
fn run_permitted_command_with_privileged_parent<'ctx, 'user>(
    plan: &ExecutionPlan<'_>,
    pam_session: pam_client::Session<'ctx, Converser<'user>>,
) -> Result<SpawnOutcome, String> {
    use nix::{
        errno::Errno,
        sys::signal::{kill, sigaction, SaFlags, SigAction, SigHandler, SigSet, Signal},
        sys::wait::{waitpid, WaitStatus},
        unistd::{fork, getpid, ForkResult},
    };
    use std::time::Duration;

    match unsafe { fork() }.map_err(|err| format!("fork: {err}"))? {
        ForkResult::Child => {
            let code = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                run_permitted_command(plan)
            })) {
                Ok(Ok(SpawnOutcome::Exit(code))) => code,
                Ok(Ok(SpawnOutcome::Signal(code))) => {
                    use nix::{
                        sys::signal::{kill, Signal},
                        unistd::getpid,
                    };

                    if let Ok(signal) = Signal::try_from(code - 128) {
                        let _ = kill(getpid(), signal);
                    }
                    code
                }
                Ok(Err(msg)) => {
                    if msg.starts_with("execvp: ") {
                        eprintln!("{msg}");
                    } else {
                        print_error(&msg);
                    }
                    1
                }
                Err(_) => {
                    print_error("Error while trying to run: panic during command execution");
                    1
                }
            };
            std::process::exit(code);
        }
        ForkResult::Parent { child } => {
            CAUGHT_SESSION_SIGNAL.store(0, Ordering::Relaxed);
            let action = SigAction::new(
                SigHandler::Handler(catch_session_signal),
                SaFlags::empty(),
                SigSet::empty(),
            );
            let old_term = unsafe { sigaction(Signal::SIGTERM, &action) }
                .map_err(|err| format!("sigaction: {err}"))?;
            let old_alrm = unsafe { sigaction(Signal::SIGALRM, &action) }
                .map_err(|err| format!("sigaction: {err}"))?;
            let old_tstp = unsafe { sigaction(Signal::SIGTSTP, &action) }
                .map_err(|err| format!("sigaction: {err}"))?;

            let outcome = loop {
                match waitpid(Some(child), None) {
                    Ok(WaitStatus::Exited(_, code)) => break Ok((code, None)),
                    Ok(WaitStatus::Signaled(_, signal, core_dumped)) => {
                        print_signal_message(signal as i32, core_dumped);
                        break Ok((128 + signal as i32, None));
                    }
                    Ok(_) => (),
                    Err(Errno::EINTR) => {
                        let caught = CAUGHT_SESSION_SIGNAL.swap(0, Ordering::Relaxed);
                        if let Ok(signal) = Signal::try_from(caught) {
                            break Ok((128 + caught, Some(signal)));
                        }
                    }
                    Err(errno) => break Err(format!("waitpid: {}", errno.desc())),
                }
            };

            restore_session_handlers(old_term, old_alrm, old_tstp);

            let (status, caught_signal) = outcome?;

            if caught_signal.is_some() {
                eprintln!("\nSession terminated, killing shell");
                let _ = kill(child, Signal::SIGTERM);
            }

            close_pam_session(pam_session)?;

            if let Some(caught_signal) = caught_signal {
                std::thread::sleep(Duration::from_secs(2));
                let _ = kill(child, Signal::SIGKILL);
                eprintln!(" ...killed.");

                let resend = if caught_signal == Signal::SIGTERM {
                    Signal::SIGTERM
                } else {
                    Signal::SIGKILL
                };
                let _ = kill(getpid(), resend);
            }

            Ok(SpawnOutcome::Exit(status))
        }
    }
}

#[cfg(auth = "pam")]
fn close_pam_session<'ctx, 'user>(
    pam_session: pam_client::Session<'ctx, Converser<'user>>,
) -> Result<(), String> {
    match pam_session.close(pam_client::Flag::NONE) {
        Ok(()) => Ok(()),
        Err(err) => Err(format!("pam_close_session: {err}")),
    }
}

#[cfg(auth = "pam")]
fn restore_session_handlers(
    old_term: nix::sys::signal::SigAction,
    old_alrm: nix::sys::signal::SigAction,
    old_tstp: nix::sys::signal::SigAction,
) {
    use nix::sys::signal::{sigaction, Signal};

    unsafe {
        let _ = sigaction(Signal::SIGTERM, &old_term);
        let _ = sigaction(Signal::SIGALRM, &old_alrm);
        let _ = sigaction(Signal::SIGTSTP, &old_tstp);
    }
}

fn print_signal_message(signal: i32, core_dumped: bool) {
    let desc = unsafe {
        let raw = libc::strsignal(signal);
        if raw.is_null() {
            format!("Signal {}", signal)
        } else {
            CStr::from_ptr(raw).to_string_lossy().into_owned()
        }
    };

    if core_dumped {
        eprintln!("{desc} (core dumped)");
    } else {
        eprintln!("{desc}");
    }
}
