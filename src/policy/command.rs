pub fn get_cmdline<A: AsRef<str>>(cmd: &str, args: &[A]) -> String {
    let mut cmdline = String::from(cmd);
    if !args.is_empty() {
        cmdline.push(' ');
        cmdline.push_str(
            &args.iter()
                .map(|arg| arg.as_ref())
                .collect::<Vec<_>>()
                .join(" "),
        );
    }
    cmdline
}
