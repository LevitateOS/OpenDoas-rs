use std::ffi::OsString;

fn program_name() -> String {
    program_name_from(std::env::args_os())
}

fn program_name_from(mut args: impl Iterator<Item = OsString>) -> String {
    args.next()
        .and_then(|value| {
            std::path::Path::new(&value)
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
        })
        .unwrap_or_else(|| String::from("doas"))
}

pub fn print_help_and_exit(code: i32) -> ! {
    let name = program_name();
    eprintln!("usage: {name} [-Lns] [-C config] [-u user] command [args]");
    std::process::exit(code);
}

pub fn print_error(msg: &str) {
    let name = program_name();
    eprintln!("{name}: {}", msg);
}

pub fn print_error_and_exit(msg: &str, code: i32) -> ! {
    print_error(msg);
    std::process::exit(code);
}

#[cfg(test)]
mod tests {
    use std::os::unix::ffi::OsStringExt;

    use super::program_name_from;

    #[test]
    fn renders_non_utf8_program_name_lossily() {
        let name =
            program_name_from([OsStringExt::from_vec(b"/usr/bin/doas\xff".to_vec())].into_iter());

        assert_eq!(name, String::from("doas\u{fffd}"));
    }
}
