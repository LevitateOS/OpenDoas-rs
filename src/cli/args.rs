use std::{collections::VecDeque, ffi::OsString};

use getopt::Opt;

use crate::platform::parse_target_uid;

use super::{
    mode::Mode,
    usage::{print_error, print_help_and_exit},
};

#[derive(Debug)]
pub enum Command {
    Execute(Execute),
    Deauth,
}

#[derive(Debug)]
pub struct Execute {
    pub interactive: bool,
    pub config_file: Option<String>,
    pub user: String,
    pub target_uid: u32,
    pub cmd: Option<String>,
    pub args: Vec<String>,
}

impl Command {
    pub fn new() -> Self {
        Self::new_from_os(std::env::args_os())
    }

    pub fn new_from(args: impl Iterator<Item = String>) -> Self {
        let mut exec_cmd = Execute {
            interactive: true,
            config_file: None,
            user: "root".into(),
            target_uid: 0,
            cmd: None,
            args: Vec::new(),
        };
        let mut deauth = false;

        let mut args: Vec<_> = args.collect();
        let mut opts = getopt::Parser::new(&args, "LnsC:u:");

        let mut exec_shell = false;
        loop {
            match opts.next() {
                None => break,
                Some(result) => match result {
                    Ok(opt) => match opt {
                        Opt('L', None) => deauth = true,
                        Opt('n', None) => exec_cmd.interactive = false,
                        Opt('s', None) => exec_shell = true,
                        Opt('C', Some(arg)) => exec_cmd.config_file = Some(arg.clone()),
                        Opt('u', Some(arg)) => {
                            exec_cmd.user = arg.clone();
                            exec_cmd.target_uid = parse_target_uid(&arg).unwrap_or_else(|err| {
                                eprintln!("doas: {err}");
                                std::process::exit(1);
                            });
                        }
                        _ => unreachable!(),
                    },
                    Err(error) => {
                        let message = error.to_string();
                        if let Some(flag) = message
                            .strip_prefix("unknown option -- '")
                            .and_then(|rest| rest.strip_suffix('\''))
                        {
                            print_error(&format!("unrecognized option: {flag}"));
                        } else {
                            print_error(&message);
                        }
                        print_help_and_exit(1);
                    }
                },
            }
        }

        let mut cmd_args = VecDeque::from(args.split_off(opts.index()));

        if exec_cmd.config_file.is_some() && exec_shell {
            print_help_and_exit(1);
        }
        if exec_shell && !cmd_args.is_empty() {
            print_help_and_exit(1);
        }

        if cmd_args.is_empty() {
            if !deauth && !exec_shell && exec_cmd.config_file.is_none() {
                print_help_and_exit(1);
            }
        } else {
            exec_cmd.cmd = Some(cmd_args.pop_front().unwrap());
            exec_cmd.args = Vec::from(cmd_args);
        }

        if deauth {
            Command::Deauth
        } else {
            Command::Execute(exec_cmd)
        }
    }

    pub fn new_from_os(args: impl Iterator<Item = OsString>) -> Self {
        Self::new_from(args.map(|arg| arg.to_string_lossy().into_owned()))
    }

    pub fn mode(&self) -> Mode {
        match self {
            Command::Deauth => Mode::Deauth,
            Command::Execute(exec) => {
                if exec.config_file.is_some() {
                    Mode::Check
                } else if exec.cmd.is_none() {
                    Mode::Shell
                } else {
                    Mode::Execute
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    use super::{Command, Execute};

    #[test]
    fn parses_non_utf8_argv_without_panicking() {
        let command = Command::new_from_os(
            [
                OsString::from("doas"),
                OsStringExt::from_vec(vec![0xff]),
                OsString::from("--flag"),
            ]
            .into_iter(),
        );

        let Command::Execute(Execute { cmd, args, .. }) = command else {
            panic!("expected execute command");
        };
        assert_eq!(cmd.as_deref(), Some("\u{fffd}"));
        assert_eq!(args, vec![String::from("--flag")]);
    }
}
