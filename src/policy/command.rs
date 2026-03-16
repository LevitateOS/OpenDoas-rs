const CMDLINE_LOG_LIMIT: usize = 2048;
const CMDLINE_LOG_BYTES: usize = CMDLINE_LOG_LIMIT - 1;

pub fn get_cmdline<A: AsRef<str>>(cmd: &str, args: &[A]) -> String {
    let mut cmdline = Vec::new();
    push_cmdline_segment(&mut cmdline, cmd.as_bytes());
    for arg in args {
        if !push_cmdline_segment(&mut cmdline, b" ") {
            break;
        }
        if !push_cmdline_segment(&mut cmdline, arg.as_ref().as_bytes()) {
            break;
        }
    }

    String::from_utf8_lossy(&cmdline).into_owned()
}

fn push_cmdline_segment(cmdline: &mut Vec<u8>, segment: &[u8]) -> bool {
    let remaining = CMDLINE_LOG_BYTES.saturating_sub(cmdline.len());
    if remaining == 0 {
        return false;
    }

    let copied = remaining.min(segment.len());
    cmdline.extend_from_slice(&segment[..copied]);
    copied == segment.len()
}

#[cfg(test)]
mod tests {
    use super::{get_cmdline, CMDLINE_LOG_BYTES};

    #[test]
    fn joins_arguments_without_truncation() {
        assert_eq!(
            get_cmdline("/bin/echo", &["hello", "world"]),
            "/bin/echo hello world"
        );
    }

    #[test]
    fn truncates_to_open_doas_log_buffer_size() {
        let arg = "y".repeat(CMDLINE_LOG_BYTES);
        let cmdline = get_cmdline("/bin/echo", &[arg.as_str()]);

        assert_eq!(cmdline.len(), CMDLINE_LOG_BYTES);
        assert!(cmdline.starts_with("/bin/echo "));
    }
}
