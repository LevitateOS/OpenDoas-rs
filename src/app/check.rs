use crate::policy::decision::Decision;

pub fn render_check_result(decision: Decision) -> (String, i32) {
    (decision.check_output(), decision.check_exit_code())
}
