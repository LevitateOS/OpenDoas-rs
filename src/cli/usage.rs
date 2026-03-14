fn program_name() -> String {
    std::env::args()
        .next()
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
