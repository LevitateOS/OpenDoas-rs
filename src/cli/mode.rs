#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Mode {
    Execute,
    Check,
    Shell,
    Deauth,
}
